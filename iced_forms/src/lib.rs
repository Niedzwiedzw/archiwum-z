use std::str::FromStr;

use chrono::{
    DateTime,
    NaiveDateTime,
    Utc,
};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum IcedFormValueError {
    #[error("Deserializing value of type [{type_name}] - {message}")]
    Deserializing {
        type_name: &'static str,
        message: String,
    },
}

pub type IcedFormValueResult<T> = Result<T, IcedFormValueError>;

pub trait IcedFormValue: Sized {
    fn serialize(&self) -> String;
    fn deserialize(val: &str) -> IcedFormValueResult<Self>;
}

static DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
// #[derive(Clone, Debug)]
// pub struct IcedFormValueResult<T>(pub T);

impl IcedFormValue for NaiveDateTime {
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

impl IcedFormValue for String {
    fn serialize(&self) -> String {
        self.clone()
    }

    fn deserialize(val: &str) -> IcedFormValueResult<Self> {
        Ok(val.to_string())
    }
}

impl IcedFormValue for uuid::Uuid {
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

// impl_iced_form!(NaiveDateTime);
// impl_iced_form!(String);
// impl_iced_form!(uuid::Uuid);

pub trait IcedForm<'a, F, Parent>: Sized + Clone
where
    F: Fn(IcedFormValueResult<Self>) -> IcedFormValueResult<Parent>,
    Parent: Clone,
{
    fn view(
        &self,
        name: &'static str,
        on_change: F,
    ) -> iced::pure::widget::Container<'a, IcedFormValueResult<Self>>;
}
use iced::pure::{
    container,
    text_input,
};

impl<'a, F, Parent> IcedForm<'a, F, Parent> for uuid::Uuid
where
    F: Fn(IcedFormValueResult<Self>) -> IcedFormValueResult<Parent>,
    Parent: Clone,
{
    fn view(
        &self,
        name: &'static str,
        on_change: F,
    ) -> iced::pure::widget::Container<'a, IcedFormValueResult<Self>> {
        container::<'a>(text_input(name, &self.serialize(), move |val: String| {
            on_change(<uuid::Uuid>::deserialize(&val))
        }))
    }
}

#[macro_export]
macro_rules! impl_iced_form {
    ($ty:ty) => {
        impl<'a, F, ParentMessage> IcedForm<'a, F, ParentMessage> for $ty
        where
            F: Fn(IcedFormValueResult<Self>) -> ParentMessage,
            ParentMessage: Clone,
        {
            fn view(
                &self,
                name: &'static str,
                on_change: impl Fn(IcedFormValueResult<Self>) -> ParentMessage,
            ) -> iced::pure::widget::Container<'a, IcedFormValueResult<Self>> {
                container::<'a>(text_input(name, &self.serialize(), move |val: String| {
                    on_change(<$ty>::deserialize(&val))
                }))
            }
        }
    };
}
// impl<'a, T> IcedForm<'a, T> for T // no idea how to get this to work for now
// where
//     T: IcedFormValue,
// {
//     type Message = T::Message;
//     fn view(
//         &self,
//         name: &'static str,
//         value: &T,
//         on_change: impl Fn(T) -> Self::Message + 'a,
//         on_error: impl Fn(IcedFormValueError) -> Self::Message + 'a,
//     ) -> iced::pure::widget::Container<'a, Self::Message> {
//         container::<'a>(text_input(
//             name,
//             &value.serialize(),
//             move |val: String| match T::deserialize(&val) {
//                 Ok(v) => on_change(v),
//                 Err(e) => on_error(e),
//             },
//         ))
//     }
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
