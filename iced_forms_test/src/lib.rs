// use iced_forms::IcedForm;
// use iced_forms_derive::IcedFormDerive;

// mod simple {
//     use super::*;

//     // #[derive(Clone, IcedFormDerive)]
//     pub struct Dog {
//         pub name: String,
//     }
// }

// mod debug {
//     #[derive(Clone)]
//     pub struct Dog {
//         pub name: String,
//     }

//     impl<'a> iced_forms::IcedForm<'a> for Dog {
//         fn view(
//             &self,
//             name: &'static str,
//             on_change: impl Fn(iced_forms::IcedFormValueResult<Self>) -> iced_forms::IcedFormValueResult<Self>
//                 + 'a,
//         ) -> iced::pure::widget::Container<'a, iced_forms::IcedFormValueResult<Self>> {
//             use iced::pure::{
//                 column,
//                 container,
//                 row,
//                 text,
//             };
//             let subform = self
//                 .name
//                 .view("ty", move |changed_field| match changed_field {
//                     Ok(value) => {
//                         let mut form = self.clone();
//                         form.name = value;
//                         on_change(iced_forms::IcedFormValueResult::<Self>::Ok(form))
//                     }
//                     Err(e) => on_change(iced_forms::IcedFormValueResult::<Self>::Err(e)),
//                 });
//             let row = row().push(text("ident")).push(subform);
//             let column = column().push(row);
//             iced::pure::container::<'a>(column)
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         let result = 2 + 2;
//         assert_eq!(result, 4);
//     }
// }
