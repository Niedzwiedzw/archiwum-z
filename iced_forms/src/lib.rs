use std::str::FromStr;

use chrono::{
    DateTime,
    NaiveDateTime,
    Utc,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IcedFormValueError {
    #[error("Deserializing value of type [{type_name}] - {message}")]
    Deserializing {
        type_name: &'static str,
        message: String,
    },
}

pub type IcedFormValueResult<T> = Result<T, IcedFormValueError>;

pub trait IcedFormValue: Sized {
    type Message;
    fn serialize(&self) -> String;
    fn deserialize(val: &str) -> IcedFormValueResult<Self>;
}

static DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
pub struct Updated<T>(T);

impl IcedFormValue for NaiveDateTime {
    type Message = Updated<Self>;
    fn serialize(&self) -> String {
        NaiveDateTime::format(self, DATE_TIME_FORMAT).to_string()
    }

    fn deserialize(val: &str) -> IcedFormValueResult<Self> {
        NaiveDateTime::parse_from_str(val, DATE_TIME_FORMAT).map_err(|e| {
            IcedFormValueError::Deserializing {
                type_name: "chrono::NaiveDateTime",
                message: e.to_string(),
            }
        })
    }
}

impl IcedFormValue for uuid::Uuid {
    type Message = Updated<Self>;
    fn serialize(&self) -> String {
        self.to_string()
    }

    fn deserialize(val: &str) -> IcedFormValueResult<Self> {
        uuid::Uuid::from_str(val).map_err(|e| IcedFormValueError::Deserializing {
            type_name: "chrono::NaiveDateTime",
            message: e.to_string(),
        })
    }
}

impl IcedFormValue for String {
    type Message = Updated<Self>;
    fn serialize(&self) -> String {
        self.clone()
    }

    fn deserialize(val: &str) -> IcedFormValueResult<Self> {
        Ok(val.to_owned())
    }
}

pub trait IcedForm<'a, T>
where
    Self::Message: Clone,
{
    type Message;
    fn view(
        &self,
        name: &'static str,
        value: &T,
        on_change: impl Fn(T) -> Self::Message + 'a,
        on_error: impl Fn(IcedFormValueError) -> Self::Message + 'a,
    ) -> iced::pure::widget::Container<'a, Self::Message>;
}
use iced::pure::{
    container,
    text_input,
};
impl<'a, T> IcedForm<'a, T> for T
where
    T: IcedFormValue,
    Self::Message: Clone,
{
    type Message = T::Message;
    fn view(
        &self,
        name: &'static str,
        value: &T,
        on_change: impl Fn(T) -> Self::Message + 'a,
        on_error: impl Fn(IcedFormValueError) -> Self::Message + 'a,
    ) -> iced::pure::widget::Container<'a, Self::Message> {
        container::<'a>(text_input(
            name,
            &value.serialize(),
            move |val: String| match T::deserialize(&val) {
                Ok(v) => on_change(v),
                Err(e) => on_error(e),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
