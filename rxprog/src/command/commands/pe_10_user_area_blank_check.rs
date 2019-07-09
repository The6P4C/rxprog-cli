use std::convert::Infallible;
use std::io;

use super::command::*;
use super::data::ErasureState;
use super::reader::*;

/// Checks if the user area is unprogrammed
#[derive(Debug)]
pub struct UserAreaBlankCheck {}

impl TransmitCommandData for UserAreaBlankCheck {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x4D,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for UserAreaBlankCheck {
    type Response = ErasureState;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorFirstByte(0xCD),
        );

        let response = reader.read_response()?;

        let state = match response {
            Ok(_) => ErasureState::Blank,
            Err(error_code) => match error_code {
                0x52 => ErasureState::NotBlank,
                _ => panic!("Unknown error code"),
            },
        };

        Ok(Ok(state))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = UserAreaBlankCheck {};
        let command_bytes = [0x4D];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx_blank() {
        let cmd = UserAreaBlankCheck {};
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(ErasureState::Blank));
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_not_blank() {
        let cmd = UserAreaBlankCheck {};
        let response_bytes = [0xCD, 0x52];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(ErasureState::NotBlank));
        assert!(is_script_complete(&mut p));
    }
}