use anyhow::anyhow;

pub mod display;
pub mod macros;
pub mod message;
pub mod registry;
pub mod types;

pub enum WlObjectId {
    Display = 1,
    Registry = 2,
    Callback = 3,
    Compositor = 4,
    ShmPool = 5,
    Shm = 6,
    Buffer = 7,
    DataOffer = 8,
    DataSource = 9,
    DataDevice = 10,
    DataDeviceManager = 11,
    Shell = 12,
    ShellSurface = 13,
    Surface = 14,
    Seat = 15,
    Pointer = 16,
    Keyboard = 17,
    Touch = 18,
    Output = 19,
    Region = 20,
    SubCompositor = 21,
    SubSurface = 22,
    Fixes = 23,
}

impl From<WlObjectId> for u32 {
    fn from(val: WlObjectId) -> Self {
        val as u32
    }
}

impl TryFrom<u32> for WlObjectId {
    type Error = anyhow::Error;
    fn try_from(value: u32) -> anyhow::Result<Self> {
        match value {
            1 => Ok(WlObjectId::Display),
            2 => Ok(WlObjectId::Registry),
            3 => Ok(WlObjectId::Callback),
            4 => Ok(WlObjectId::Compositor),
            5 => Ok(WlObjectId::ShmPool),
            6 => Ok(WlObjectId::Shm),
            7 => Ok(WlObjectId::Buffer),
            8 => Ok(WlObjectId::DataOffer),
            9 => Ok(WlObjectId::DataSource),
            10 => Ok(WlObjectId::DataDevice),
            11 => Ok(WlObjectId::DataDeviceManager),
            12 => Ok(WlObjectId::Shell),
            13 => Ok(WlObjectId::ShellSurface),
            14 => Ok(WlObjectId::Surface),
            15 => Ok(WlObjectId::Seat),
            16 => Ok(WlObjectId::Pointer),
            17 => Ok(WlObjectId::Keyboard),
            18 => Ok(WlObjectId::Touch),
            19 => Ok(WlObjectId::Output),
            20 => Ok(WlObjectId::Region),
            21 => Ok(WlObjectId::SubCompositor),
            22 => Ok(WlObjectId::SubSurface),
            23 => Ok(WlObjectId::Fixes),
            _ => Err(anyhow!("WlObjectID: Invalid id")),
        }
    }
}
