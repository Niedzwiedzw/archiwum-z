use std::{fmt::Display, str::FromStr, sync::Arc};

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de::DeserializeOwned, Serialize};
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
#[derive(Clone, Debug)]
pub enum SelectorSegment {
    ArrayIndex(usize),
    ObjectField(String),
}

#[derive(Debug, Clone, Default)]
pub struct Selector(Vec<SelectorSegment>);

impl Selector {
    pub fn push(&self, segment: SelectorSegment) -> Self {
        let mut new = self.0.clone();
        new.push(segment);
        Self(new)
    }
    pub fn pop(&self) -> Self {
        let mut new = self.0.clone();
        new.pop();
        Self(new)
    }
    pub fn empty() -> Self {
        Self(Default::default())
    }
}

impl Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.last() {
            Some(segment) => segment.fmt(f),
            None => "".fmt(f),
        }
    }
}

impl Display for SelectorSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectorSegment::ArrayIndex(i) => i.fmt(f),
            SelectorSegment::ObjectField(key) => key.fmt(f),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpdatedMessage {
    pub selector: Selector,
    pub value: IcedFormValueResult<serde_json::Value>,
}
// pub type Key = vec![];
// pub struct Message {
//     key:
// }
// // pub trait IcedFormValue: Sized {
//     fn serialize(&self) -> String;
//     fn deserialize(val: &str) -> IcedFormValueResult<Self>;
// }

// static DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
// // #[derive(Clone, Debug)]
// // pub struct IcedFormValueResult<T>(pub T);

// impl IcedFormValue for NaiveDateTime {
//     fn serialize(&self) -> String {
//         NaiveDateTime::format(self, DATE_TIME_FORMAT).to_string()
//     }

//     fn deserialize(val: &str) -> IcedFormValueResult<Self> {
//         NaiveDateTime::parse_from_str(val, DATE_TIME_FORMAT).map_err(|e| {
//             IcedFormValueError::Deserializing {
//                 type_name: "chrono::NaiveDateTime",
//                 message: e.to_string(),
//             }
//         })
//     }
// }

// impl IcedFormValue for String {
//     fn serialize(&self) -> String {
//         self.clone()
//     }

//     fn deserialize(val: &str) -> IcedFormValueResult<Self> {
//         Ok(val.to_string())
//     }
// }

// impl IcedFormValue for uuid::Uuid {
//     fn serialize(&self) -> String {
//         self.to_string()
//     }

//     fn deserialize(val: &str) -> IcedFormValueResult<Self> {
//         uuid::Uuid::from_str(val).map_err(|e| IcedFormValueError::Deserializing {
//             type_name: "chrono::NaiveDateTime",
//             message: e.to_string(),
//         })
//     }
// }

// impl_iced_form!(NaiveDateTime);
// impl_iced_form!(String);
// impl_iced_form!(uuid::Uuid);

use iced::{
    pure::{checkbox, column, container, text, text_input},
    Length,
};

#[derive(Clone, Debug)]
pub struct IcedFormBuffer<T>(pub T);

impl<T> IcedFormBuffer<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    pub fn to_buffer(&self) -> IcedFormValueResult<serde_json::Value> {
        serde_json::to_string(&self.0)
            .and_then(|v| serde_json::from_str(&v))
            .map_err(|e| IcedFormValueError::Deserializing {
                type_name: std::any::type_name::<T>(),
                message: e.to_string(),
            })
    }

    pub fn from_buffer(value: &serde_json::Value) -> IcedFormValueResult<Self> {
        serde_json::to_string(value)
            .and_then(|value| serde_json::from_str(&value))
            .map_err(|e| IcedFormValueError::Deserializing {
                type_name: std::any::type_name::<T>(),
                message: e.to_string(),
            })
            .map(Self)
    }
}

pub fn to_value<T: Clone + Serialize + DeserializeOwned>(
    val: T,
) -> IcedFormValueResult<serde_json::Value> {
    serde_json::to_string(&val)
        .and_then(|v| serde_json::from_str(&v))
        .map_err(|e| IcedFormValueError::Deserializing {
            type_name: std::any::type_name::<T>(),
            message: e.to_string(),
        })
}

pub fn from_value<T: Clone + Serialize + DeserializeOwned>(
    value: &serde_json::Value,
) -> IcedFormValueResult<T> {
    serde_json::to_string(value)
        .and_then(|value| serde_json::from_str(&value))
        .map_err(|e| IcedFormValueError::Deserializing {
            type_name: std::any::type_name::<T>(),
            message: e.to_string(),
        })
}

// impl<'a, F, T> IcedForm<'a, F> for IcedFormBuffer<T>
// where
//     F: Clone + Fn(IcedFormValueResult<Self>) -> IcedFormValueResult<Self> + 'a,
//     T: Clone + Serialize + DeserializeOwned,
// {
//     fn view(
//         &'a self,
//         on_change: F,
//         selector: Selector,
//     ) -> iced::pure::widget::Container<'a, IcedFormValueResult<Self>> {
//         let on_update = ||
//         self.to_buffer()
//             .expect("failed to display form")
//             .view(|updated| , selector)
//     }
// }

pub trait IcedForm<'a, Message>: Sized + Clone
where
    Message: Clone + 'a,
{
    fn view(
        &'a self,
        on_change: Arc<dyn Fn(IcedFormValueResult<serde_json::Value>) -> Message + 'a>,
        selector: Selector,
    ) -> iced::pure::widget::Container<'a, Message>;
}

impl<'a, Message> IcedForm<'a, Message> for serde_json::Value
where
    Message: Clone + 'a,
{
    fn view(
        &'a self,
        on_change: Arc<dyn Fn(IcedFormValueResult<serde_json::Value>) -> Message + 'a>,
        selector: Selector,
    ) -> iced::pure::widget::Container<'a, Message> {
        match self {
            serde_json::Value::Null => container(text(&format!("{selector}"))),
            serde_json::Value::Bool(value) => container(
                checkbox(&format!("{selector}"), *value, move |value| {
                    on_change(to_value(value))
                })
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill),
            serde_json::Value::Number(value) => container(
                text_input(
                    &format!("{selector}"),
                    &serde_json::to_string(&serde_json::Value::Number(value.clone()))
                        .expect("failed to serialize a number?"),
                    move |value| {
                        on_change(serde_json::from_str(&value).map_err(|e| {
                            IcedFormValueError::Deserializing {
                                type_name: "Number",
                                message: e.to_string(),
                            }
                        }))
                    },
                )
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill),
            serde_json::Value::String(value) => container(
                text_input(&format!("{selector}"), value, move |value| {
                    on_change(Ok(serde_json::Value::String(value)))
                })
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill),
            serde_json::Value::Array(values) => container(values.iter().enumerate().fold(
                column(),
                move |acc, (index, value)| {
                    let on_change = on_change.clone();
                    acc.push(value.view(
                        Arc::new(move |value| match value {
                            Ok(value) => {
                                let mut new = values.clone();
                                new.insert(index, value);
                                on_change(Ok(serde_json::Value::Array(new)))
                            }
                            Err(e) => on_change(Err(e)),
                        }),
                        selector.push(SelectorSegment::ArrayIndex(index)),
                    ))
                },
            ))
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill),
            serde_json::Value::Object(values) => {
                container(values.iter().fold(column(), move |acc, (key, value)| {
                    let on_change = on_change.clone();
                    acc.push(value.view(
                        Arc::new(move |value| match value {
                            Ok(value) => {
                                let mut new = values.clone();
                                new.insert(key.to_string(), value);
                                on_change(Ok(serde_json::Value::Object(new)))
                            }
                            Err(e) => on_change(Err(e)),
                        }),
                        selector.push(SelectorSegment::ObjectField(key.to_string())),
                    ))
                }))
                .padding(10)
                .width(Length::Fill)
                .height(Length::Fill)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_serialization() {}
}
// impl<'a, F, Parent> IcedForm<'a, F, Parent> for uuid::Uuid
// where
//     F: Fn(IcedFormValueResult<Self>) -> IcedFormValueResult<Parent>,
//     Parent: Clone,
// {
//     fn view(
//         &self,
//         name: &'static str,
//         on_change: F,
//     ) -> iced::pure::widget::Container<'a, IcedFormValueResult<Self>> {
//         container::<'a>(text_input(name, &self.serialize(), move |val: String| {
//             on_change(<uuid::Uuid>::deserialize(&val))
//         }))
//     }
// }

// #[macro_export]
// macro_rules! impl_iced_form {
//     ($ty:ty) => {
//         impl<'a, F, ParentMessage> IcedForm<'a, F, ParentMessage> for $ty
//         where
//             F: Fn(IcedFormValueResult<Self>) -> ParentMessage,
//             ParentMessage: Clone,
//         {
//             fn view(
//                 &self,
//                 name: &'static str,
//                 on_change: impl Fn(IcedFormValueResult<Self>) -> ParentMessage,
//             ) -> iced::pure::widget::Container<'a, IcedFormValueResult<Self>> {
//                 container::<'a>(text_input(name, &self.serialize(), move |val: String| {
//                     on_change(<$ty>::deserialize(&val))
//                 }))
//             }
//         }
//     };
// }
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
