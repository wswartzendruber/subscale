mod seg {

    use std::{
        io::{Error as IoError, Read},
        result::Result,
    };
    use byteorder::{BigEndian, ReadBytesExt};
    use thiserror::Error as ThisError;

    pub type SegResult<T> = Result<T, SegError>;

    #[derive(ThisError, Debug)]
    pub enum SegError {
        #[error("segment IO error")]
        PrematureEof {
            #[from]
            source: IoError,
        },
        #[error("segment has unrecognized magic number")]
        UnrecognizedMagicNumber,
        #[error("segment has unrecognized kind")]
        UnrecognizedKind,
        #[error("presentation control segment has unrecognized frame rate")]
        UnrecognizedFrameRate,
        #[error("presentation control segment has unrecognized composition state")]
        UnrecognizedCompositionState,
        #[error("presentation control segment has unrecognized palette update flag")]
        UnrecognizedPaletteUpdateFlag,
        #[error("composition object has unrecognized cropped flag")]
        UnrecognizedCroppedFlag,
    }

    pub enum CompState {
        Normal,
        AcquisitionPoint,
        EpochStart,
    }

    pub struct CompObj {
        obj_id: u16,
        win_id: u8,
        cropped: bool,
        x: u16,
        y: u16,
        x_cropped: Option<u16>,
        y_cropped: Option<u16>,
        width_cropped: Option<u16>,
        height_cropped: Option<u16>,
    }

    pub struct PresCompSeg {
        width: u16,
        height: u16,
        comp_num: u16,
        comp_state: CompState,
        pal_update: bool,
        pal_id: u8,
        comp_objs: Vec<CompObj>,
    }

    pub enum SegBody {
        PresComp(PresCompSeg),
    }

    pub struct Seg {
        pts: u32,
        dts: u32,
        body: SegBody,
    }

    pub trait ReadExt {
        fn read_seg(&mut self) -> SegResult<Seg>;
    }

    impl ReadExt for dyn Read {

        fn read_seg(&mut self) -> SegResult<Seg> {

            if self.read_u16::<BigEndian>()? != 0x5047 {
                return Err(SegError::UnrecognizedMagicNumber)
            }

            let pts = self.read_u32::<BigEndian>()?;
            let dts = self.read_u32::<BigEndian>()?;
            let body = match self.read_u8()? {
                //0x14 => SegType::Pds,
                //0x15 => SegType::Ods,
                0x16 => SegBody::PresComp(parse_pcs(self)?),
                //0x17 => SegType::Wds,
                //0x80 => SegType::End,
                _ => return Err(SegError::UnrecognizedKind),
            };

            Ok(Seg { pts, dts, body })
        }
    }

    fn parse_pcs(input: &mut dyn Read) -> SegResult<PresCompSeg> {

        let size = input.read_u16::<BigEndian>()? as usize;
        let width = input.read_u16::<BigEndian>()?;
        let height = input.read_u16::<BigEndian>()?;

        if input.read_u8()? != 0x10 {
            return Err(SegError::UnrecognizedFrameRate)
        }

        let comp_num = input.read_u16::<BigEndian>()?;
        let comp_state = match input.read_u8()? {
            0x00 => CompState::Normal,
            0x40 => CompState::AcquisitionPoint,
            0x80 => CompState::EpochStart,
            _ => return Err(SegError::UnrecognizedCompositionState),
        };
        let pal_update = match input.read_u8()? {
            0x00 => false,
            0x80 => true,
            _ => return Err(SegError::UnrecognizedPaletteUpdateFlag),
        };
        let pal_id = input.read_u8()?;
        let comp_obj_count = input.read_u8()? as usize;
        let mut comp_objs = Vec::new();

        for _ in 0..comp_obj_count {

            let obj_id = input.read_u16::<BigEndian>()?;
            let win_id = input.read_u8()?;
            let cropped = match input.read_u8()? {
                0x40 => true,
                0x00 => false,
                _ => return Err(SegError::UnrecognizedCroppedFlag),
            };
            let x = input.read_u16::<BigEndian>()?;
            let y = input.read_u16::<BigEndian>()?;
            let (
                x_cropped,
                y_cropped,
                width_cropped,
                height_cropped,
            ) = if cropped {
                (
                    Some(input.read_u16::<BigEndian>()?),
                    Some(input.read_u16::<BigEndian>()?),
                    Some(input.read_u16::<BigEndian>()?),
                    Some(input.read_u16::<BigEndian>()?),
                )
            } else {
                (
                    None,
                    None,
                    None,
                    None,
                )
            };

            comp_objs.push(
                CompObj {
                    obj_id,
                    win_id,
                    cropped,
                    x,
                    y,
                    x_cropped,
                    y_cropped,
                    width_cropped,
                    height_cropped,
                }
            );
        }

        Ok(
            PresCompSeg {
                width,
                height,
                comp_num,
                comp_state,
                pal_update,
                pal_id,
                comp_objs,
            }
        )
    }
}
