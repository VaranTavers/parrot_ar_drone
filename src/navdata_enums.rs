pub enum DemoOptions {
    Default,
    Init,
    Landed,
    Flying,
    Hovering,
    Test,
    TransTakeoff,
    TransGoFix,
    TransLanding,
    TransLooping,
    TransNoVision,
    NumState,
    BatteryPercentage,
    Theta,
    Phi,
    Psi,
    Altitude,
    Vx,
    Vy,
    Vz,
    NumFrames,
    DetectionCameraRot(i32, i32),
    DetectionCameraTrans(i32),
    DetectionTagIndex,
    DetectionCameraType,
    DroneCameraRot(i32, i32),
    DroneCameraTrans(i32)
}

pub enum NavDataOption {
    Header,
    Demo(DemoOptions)
}

