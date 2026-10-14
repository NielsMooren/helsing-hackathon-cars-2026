# Team SNR

Team members: Roy, Simon, and Niels

## Car-control interface

`CarController` is the interface the forthcoming OpenCV controller uses to
continuously command the car. It owns the 120 ms command heartbeat and accepts a
replacement command or stop at any time.

    use helsing_hackathon_cars_2026::{car::CarController, data};

    let car = CarController::new(data::CAR_ENDPOINT, data::CAR_TOKEN)?;
    car.drive(0.4).await?;
    car.turn_right(0.2).await?;
    car.stop().await?;

The interface calls the car endpoint with the configured token and keeps
resending the active command until it is replaced or stopped. `drive` accepts
the complete protocol range from `-1.0` to `1.0`; negative values drive
backwards.

`turn_left(speed)` sends positive speed with `flip: true`; `turn_right(speed)`
sends negative speed with `flip: true`. This matches the observed car behavior:
backwards flipped movement turns right, while forwards flipped movement turns
left. `command(speed, flip)` remains available when the vision controller needs
direct protocol access.

The interactive CLI has been removed.
