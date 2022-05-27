mod colors {
    type Color = [f32; 3];
    pub static BLACK: Color = [0., 0., 0.];
    pub static WHITE: Color = [1., 1., 1.];
}
use std::{
    path::PathBuf,
    sync::Arc,
};

use futures::FutureExt;
use iced::{
    alignment,
    pure::{
        widget::{
            Button,
            Column,
            Container,
            Row,
            Scrollable,
            Text,
            TextInput,
        },
        Application,
        Element,
    },
    Alignment,
    Command,
    Length,
};
use iced_forms::IcedFormValueResult;
use tracing::error;

use crate::{
    db::RepairContractEntry,
    models::RepairContract,
};

use super::*;

#[derive(Debug, Clone)]
pub enum Mode {
    Index,
    ViewingEntries,
    CreateNewRepairContract {
        form: RepairContract,
        buffer: IcedFormValueResult<serde_json::Value>,
    },
}
pub struct ArchiwumZ {
    pub db: crate::db::Database,
    pub mode: Mode,
    pub repair_contract_entries_buffer: Vec<RepairContractEntry>,
}

mod local_messages {
    use serde_json::Value;

    use super::*;
    #[derive(Debug, Clone)]
    pub enum CreateRepairContract {
        FormUpdated(IcedFormValueResult<Value>),
    }
}

use local_messages::*;

#[derive(Debug, Clone)]
pub enum Message {
    SwitchMode(Mode),
    RefreshRepairContracts,
    RepairContractsRefreshed(Arc<Result<Vec<RepairContractEntry>>>),
    CreateRepairContract(CreateRepairContract),
}

mod custom_widgets {
    use super::*;
    pub fn repair_contract_entry_list_item(
        repair_contract_entry: &RepairContractEntry,
    ) -> Container<Message> {
        Container::new(Text::new(repair_contract_entry.model.id.to_string()))
    }
}

mod pages {

    use iced::pure::text;
    use iced_forms::{
        IcedForm,
        IcedFormValueResult,
    };

    use crate::db::FillForm;

    use super::*;
    pub fn index<'a>() -> Column<'a, Message> {
        Column::new()
            .max_width(800)
            .spacing(20)
            .align_items(Alignment::Center)
    }

    pub fn create_new_contract_form<'a>(
        repair_contract_entries: &'a [RepairContractEntry],
        form: &'a RepairContract,
        buffer: &'a IcedFormValueResult<serde_json::Value>,
    ) -> Column<'a, Message> {
        // let with_title = |text: &'static str, element| {
        //     Row::new()
        //         .spacing(30)
        //         .align_items(Alignment::Start)
        //         .push(Text::new(text).width(Length::Units(100)))
        //         .push(element)
        // };
        let form: Element<'a, _> = match buffer {
            Ok(form) => form
                .view(
                    move |v| Message::CreateRepairContract(CreateRepairContract::FormUpdated(v)),
                    Default::default(),
                )
                .into(),
            Err(e) => text(e.to_string()).into(),
        };
        Column::new()
            .max_width(800)
            .spacing(20)
            .align_items(Alignment::Center)
            .push(form)
    }

    pub fn contracts_list(
        repair_contract_entries: &'_ [RepairContractEntry],
    ) -> Container<'_, Message> {
        let entries = repair_contract_entries.iter().fold(
            Column::new()
                .max_width(800)
                .spacing(20)
                .align_items(Alignment::Start),
            |acc, next| acc.push(custom_widgets::repair_contract_entry_list_item(next)),
        );
        Container::new(Scrollable::new(entries))
    }
}

impl Application for ArchiwumZ {
    type Executor = iced::executor::Default;

    type Message = Message;

    type Flags = ();

    fn new(_: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let base_dir = crate::filesystem::base_directory()
            .expect("nie udało się stworzyć aplikacji")
            .join("archiwum");
        let db = crate::db::Database::new(base_dir);
        (
            Self {
                db,
                mode: Mode::Index,
                repair_contract_entries_buffer: vec![],
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Archiwum Z".to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::SwitchMode(mode) => {
                self.mode = mode;
            }
            Message::RefreshRepairContracts => {
                let db = self.db.clone();
                return Command::perform(
                    async move { db.get_entries().map(Arc::new).await },
                    Message::RepairContractsRefreshed,
                );
            }
            Message::RepairContractsRefreshed(res) => match res.as_ref() {
                Ok(contracts) => self.repair_contract_entries_buffer = contracts.clone(),
                Err(e) => error!("{e:#?}"),
            },
            Message::CreateRepairContract(message) => match message {
                local_messages::CreateRepairContract::FormUpdated(updated) => {
                    match &mut self.mode {
                        Mode::Index => todo!(),
                        Mode::ViewingEntries => todo!(),
                        Mode::CreateNewRepairContract { form, buffer } => *buffer = updated,
                    }
                }
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let title = Text::new("Archiwum Z")
            .width(iced::Length::Fill)
            .size(100)
            .color(colors::BLACK)
            .horizontal_alignment(alignment::Horizontal::Center);

        let navigation = Row::new()
            .push(iced::pure::button("Archiwum Z").on_press(Message::SwitchMode(Mode::Index)))
            .push(
                iced::pure::button("Utwórz zlecenie").on_press(Message::SwitchMode({
                    let form = RepairContract::default();
                    Mode::CreateNewRepairContract {
                        form: form.clone(),
                        buffer: iced_forms::to_value(form),
                    }
                })),
            )
            .push(
                iced::pure::button("zlecenia").on_press(Message::SwitchMode(Mode::ViewingEntries)),
            );
        let navbar = Row::new().push(title).push(navigation);
        let page: Element<'_, _> = match &self.mode {
            Mode::Index => pages::index().into(),
            Mode::ViewingEntries => {
                pages::contracts_list(&self.repair_contract_entries_buffer).into()
            }
            Mode::CreateNewRepairContract { form, buffer } => {
                pages::create_new_contract_form(&self.repair_contract_entries_buffer, form, buffer)
                    .into()
            }
        };

        let global_controls =
            Row::new().push(Button::new("odśwież").on_press(Message::RefreshRepairContracts));
        let content = Column::new()
            .max_width(800)
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Row::new().push(navbar).push(global_controls))
            .push(page);
        let app = Container::new(content)
            .width(Length::Fill)
            .center_x()
            .center_y();
        app.into()
    }
}
