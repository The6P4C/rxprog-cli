use std::io;

use super::command::*;
use super::reader::*;

/// Select a device
#[derive(Debug)]
pub struct DeviceSelection {
    /// The 4 character device code of the device to select
    pub device_code: String,
}

impl TransmitCommandData for DeviceSelection {
    fn command_data(&self) -> CommandData {
        assert_eq!(self.device_code.len(), 4);

        CommandData {
            opcode: 0x10,
            has_size_field: true,
            payload: self.device_code.bytes().collect(),
        }
    }
}

/// Error preventing successful device selection
#[derive(Debug, PartialEq)]
pub enum DeviceSelectionError {
    /// Command checksum validation failed
    Checksum,
    /// Invalid device code
    DeviceCode,
}

impl Receive for DeviceSelection {
    type Response = ();
    type Error = DeviceSelectionError;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorFirstByte(0x90),
        );

        let response = reader.read_response()?;

        Ok(match response {
            Ok(_) => Ok(()),
            Err(error_code) => Err(match error_code {
                0x11 => DeviceSelectionError::Checksum,
                0x21 => DeviceSelectionError::DeviceCode,
                _ => panic!("Unknown error code"),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = DeviceSelection {
            device_code: "DEV1".to_string(),
        };
        let command_bytes = [0x10, 0x04, 0x44, 0x45, 0x56, 0x31, 0xDC];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx_success() {
        let cmd = DeviceSelection {
            device_code: "DEV1".to_string(),
        };
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(()));
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = DeviceSelection {
            device_code: "DEV1".to_string(),
        };
        let response_bytes = [0x90, 0x21];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Err(DeviceSelectionError::DeviceCode));
        assert!(is_script_complete(&mut p));
    }
}