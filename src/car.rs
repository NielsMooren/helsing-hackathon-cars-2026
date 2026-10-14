use std::time::{Duration, Instant};

use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use tokio::sync::{mpsc, oneshot};

const COMMAND_INTERVAL: Duration = Duration::from_millis(120);

type CommandResult = Result<(), String>;

#[derive(Clone)]
pub struct CarController {
    commands: mpsc::Sender<ControlCommand>,
}

struct ControlCommand {
    action: Action,
    reply: oneshot::Sender<CommandResult>,
}

#[derive(Clone, Copy)]
enum Action {
    Drive { speed: f32, flip: bool },
    Stop,
}

struct CarClient {
    client: Client,
    endpoint: String,
    token: String,
    last_command_at: Option<Instant>,
}

impl CarController {
    pub fn new(endpoint: &str, token: &str) -> Result<Self, String> {
        let endpoint = normalize_endpoint(endpoint)?;
        if token.len() != 6 || !token.chars().all(|character| character.is_ascii_digit()) {
            return Err("car token must be six digits".to_owned());
        }

        let (commands, receiver) = mpsc::channel(16);
        tokio::spawn(run_controller(
            CarClient::new(endpoint, token.to_owned()),
            receiver,
        ));
        Ok(Self { commands })
    }

    pub async fn drive(&self, speed: f32) -> CommandResult {
        self.command(speed, false).await
    }

    pub async fn turn_left(&self, speed: f32) -> CommandResult {
        self.command(speed.abs(), true).await
    }

    pub async fn turn_right(&self, speed: f32) -> CommandResult {
        self.command(-speed.abs(), true).await
    }

    pub async fn command(&self, speed: f32, flip: bool) -> CommandResult {
        validate_speed(speed)?;
        self.send(Action::Drive { speed, flip }).await
    }

    pub async fn stop(&self) -> CommandResult {
        self.send(Action::Stop).await
    }

    async fn send(&self, action: Action) -> CommandResult {
        let (reply, result) = oneshot::channel();
        self.commands
            .send(ControlCommand { action, reply })
            .await
            .map_err(|_| "car controller stopped unexpectedly".to_owned())?;
        result
            .await
            .map_err(|_| "car controller stopped before replying".to_owned())?
    }
}

impl CarClient {
    fn new(endpoint: String, token: String) -> Self {
        Self {
            client: Client::new(),
            endpoint,
            token,
            last_command_at: None,
        }
    }

    async fn send(&mut self, speed: f32, flip: bool) -> CommandResult {
        if let Some(last_command_at) = self.last_command_at {
            let elapsed = last_command_at.elapsed();
            if elapsed < COMMAND_INTERVAL {
                tokio::time::sleep(COMMAND_INTERVAL - elapsed).await;
            }
        }

        let response = self
            .client
            .put(&self.endpoint)
            .header(AUTHORIZATION, &self.token)
            .header(CONTENT_TYPE, "application/json")
            .body(format!(r#"{{"speed":{speed},"flip":{flip}}}"#))
            .send()
            .await
            .map_err(|error| format!("car request failed: {error}"))?;
        self.last_command_at = Some(Instant::now());

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("car returned HTTP {}", response.status()))
        }
    }
}

async fn run_controller(mut car: CarClient, mut commands: mpsc::Receiver<ControlCommand>) {
    let mut active_drive = None;

    loop {
        if let Some((speed, flip)) = active_drive {
            tokio::select! {
                command = commands.recv() => match command {
                    Some(command) => apply_command(&mut car, &mut active_drive, command).await,
                    None => break,
                },
                _ = tokio::time::sleep(COMMAND_INTERVAL) => {
                    if let Err(error) = car.send(speed, flip).await {
                        eprintln!("{error}");
                    }
                },
            }
        } else {
            match commands.recv().await {
                Some(command) => apply_command(&mut car, &mut active_drive, command).await,
                None => break,
            }
        }
    }

    if let Err(error) = car.send(0.0, false).await {
        eprintln!("could not send final stop command: {error}");
    }
}

async fn apply_command(
    car: &mut CarClient,
    active_drive: &mut Option<(f32, bool)>,
    command: ControlCommand,
) {
    let result = match command.action {
        Action::Drive { speed, flip } => {
            *active_drive = Some((speed, flip));
            car.send(speed, flip).await
        }
        Action::Stop => {
            *active_drive = None;
            car.send(0.0, false).await
        }
    };
    let _ = command.reply.send(result);
}

fn normalize_endpoint(input: &str) -> Result<String, String> {
    let input = input.trim().trim_end_matches('/');
    if input.is_empty() {
        return Err("car endpoint cannot be empty".to_owned());
    }
    if input.starts_with("http://") || input.starts_with("https://") {
        Ok(input.to_owned())
    } else {
        Ok(format!("http://{input}"))
    }
}

fn validate_speed(speed: f32) -> CommandResult {
    if !speed.is_finite() || !(-1.0..=1.0).contains(&speed) {
        return Err("speed must be a finite value from -1.0 through 1.0".to_owned());
    }
    Ok(())
}
