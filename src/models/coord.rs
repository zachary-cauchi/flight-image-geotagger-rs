use exif::Value;
use num::{FromPrimitive, Rational32};

use super::result::{GTError, GTResult};

pub struct Converter {}

impl Converter {
    pub fn try_coord_to_dms(coord: f64) -> Option<(u32, u32, Rational32)> {
        let degrees = coord.floor();
        let part = (coord - degrees) * 60.0;
        let minutes = part.floor();
        let seconds = (part - minutes) * 60.0;
        let seconds = Rational32::from_f64(seconds)?;

        Some((degrees as u32, minutes as u32, seconds))
    }

    pub fn try_coord_to_exif_value(coord: f64) -> GTResult<Value> {
        let (degrees, minutes, seconds) = Self::try_coord_to_dms(coord).ok_or(
            GTError::Conversion("Failed to convert to DMS format.".to_string()),
        )?;

        let degrees = exif::Rational::from((degrees, 1));
        let minutes = exif::Rational::from((minutes, 1));
        let seconds = exif::Rational::from((*seconds.numer() as u32, *seconds.denom() as u32));

        Ok(Value::Rational(vec![degrees, minutes, seconds]))
    }
}
