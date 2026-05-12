use cosmic::app::{Core, Settings as AppSettings};
use cosmic::iced::{Alignment, Length};
use cosmic::{Action, Application, ApplicationExt, Element, Task, executor, widget};
use cosmic_weekly_commits::weekly;

#[derive(Clone, Debug)]
enum Message {
    ServiceSelected(usize),
    ThemeSelected(usize),
    ColorModeSelected(usize),
    WeekStartSelected(usize),
    UsernameChanged(String),
    TokenChanged(String),
    InstanceChanged(String),
    IntervalChanged(String),
    CurrentWeekChanged(bool),
    HighlightTodayChanged(bool),
    Saved,
}

struct SettingsApp {
    core: Core,
    settings: weekly::Settings,
    interval_text: String,
    saved_notice: bool,
}

impl Application for SettingsApp {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "dev.funinkina.weekly-commits.cosmic.settings";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Action<Self::Message>>) {
        let settings = weekly::load_settings();
        let mut app = Self {
            core,
            interval_text: settings.refresh_interval.to_string(),
            settings,
            saved_notice: false,
        };
        let task = app.update_title();
        (app, task)
    }

    fn update(&mut self, message: Self::Message) -> Task<Action<Self::Message>> {
        self.saved_notice = false;
        match message {
            Message::ServiceSelected(index) => {
                if let Some(value) = weekly::ServiceType::ALL.get(index) {
                    self.settings.service_type = *value;
                }
            }
            Message::ThemeSelected(index) => {
                if let Some(value) = weekly::ThemeName::ALL.get(index) {
                    self.settings.theme_name = *value;
                }
            }
            Message::ColorModeSelected(index) => {
                if let Some(value) = weekly::ColorMode::ALL.get(index) {
                    self.settings.color_mode = *value;
                }
            }
            Message::WeekStartSelected(index) => {
                if let Some(value) = weekly::WeekdayStart::ALL.get(index) {
                    self.settings.week_start_day = *value;
                }
            }
            Message::UsernameChanged(value) => self.settings.github_username = value,
            Message::TokenChanged(value) => self.settings.github_token = value,
            Message::InstanceChanged(value) => self.settings.custom_instance_url = value,
            Message::IntervalChanged(value) => {
                self.interval_text = value;
                if let Ok(seconds) = self.interval_text.parse::<u64>() {
                    self.settings.refresh_interval = seconds.max(60);
                }
            }
            Message::CurrentWeekChanged(value) => self.settings.show_current_week_only = value,
            Message::HighlightTodayChanged(value) => self.settings.highlight_current_day = value,
            Message::Saved => {
                let _ = weekly::save_settings(&self.settings);
                self.saved_notice = true;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let service_labels: Vec<&'static str> = weekly::ServiceType::ALL
            .iter()
            .map(|value| value.label())
            .collect();
        let theme_labels: Vec<&'static str> = weekly::ThemeName::ALL
            .iter()
            .map(|value| value.label())
            .collect();
        let color_mode_labels: Vec<&'static str> = weekly::ColorMode::ALL
            .iter()
            .map(|value| value.label())
            .collect();
        let week_labels: Vec<&'static str> = weekly::WeekdayStart::ALL
            .iter()
            .map(|value| value.label())
            .collect();

        let content = widget::Column::new()
            .spacing(14)
            .padding(24)
            .max_width(560)
            .push(widget::text::title3("Weekly Commits"))
            .push(row(
                "Service",
                widget::dropdown(
                    service_labels,
                    selected_index(&weekly::ServiceType::ALL, self.settings.service_type),
                    Message::ServiceSelected,
                )
                .into(),
            ))
            .push(row(
                "Username",
                widget::text_input("Username", &self.settings.github_username)
                    .on_input(Message::UsernameChanged)
                    .into(),
            ))
            .push(row(
                "Token",
                widget::secure_input(
                    "Personal access token",
                    &self.settings.github_token,
                    None,
                    true,
                )
                .on_input(Message::TokenChanged)
                .into(),
            ))
            .push(row(
                "Instance URL",
                widget::text_input(
                    "https://gitlab.com or https://gitea.example.com",
                    &self.settings.custom_instance_url,
                )
                .on_input(Message::InstanceChanged)
                .into(),
            ))
            .push(row(
                "Refresh interval",
                widget::text_input("Seconds", &self.interval_text)
                    .on_input(Message::IntervalChanged)
                    .into(),
            ))
            .push(row(
                "Theme",
                widget::dropdown(
                    theme_labels,
                    selected_index(&weekly::ThemeName::ALL, self.settings.theme_name),
                    Message::ThemeSelected,
                )
                .into(),
            ))
            .push(row(
                "Color mode",
                widget::dropdown(
                    color_mode_labels,
                    selected_index(&weekly::ColorMode::ALL, self.settings.color_mode),
                    Message::ColorModeSelected,
                )
                .into(),
            ))
            .push(row(
                "Week start",
                widget::dropdown(
                    week_labels,
                    selected_index(&weekly::WeekdayStart::ALL, self.settings.week_start_day),
                    Message::WeekStartSelected,
                )
                .into(),
            ))
            .push(
                widget::checkbox(self.settings.show_current_week_only)
                    .label("Show current week only")
                    .on_toggle(Message::CurrentWeekChanged),
            )
            .push(
                widget::checkbox(self.settings.highlight_current_day)
                    .label("Highlight current day")
                    .on_toggle(Message::HighlightTodayChanged),
            )
            .push(
                widget::Row::new()
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .push(widget::button::suggested("Save").on_press(Message::Saved))
                    .push(if self.saved_notice {
                        widget::text::body("Saved")
                    } else {
                        widget::text::body("")
                    }),
            );

        widget::container(content)
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .into()
    }
}

impl SettingsApp {
    fn update_title(&mut self) -> Task<Action<Message>> {
        let title = "Weekly Commits Settings".to_string();
        self.set_header_title(title.clone());
        self.set_window_title(title, self.core.main_window_id().unwrap())
    }
}

fn row<'a>(label: &'static str, control: Element<'a, Message>) -> Element<'a, Message> {
    widget::Row::new()
        .spacing(16)
        .align_y(Alignment::Center)
        .push(widget::text::body(label).width(Length::Fixed(140.0)))
        .push(widget::container(control).width(Length::Fill))
        .into()
}

fn selected_index<T: PartialEq>(slice: &[T], value: T) -> Option<usize> {
    slice.iter().position(|item| *item == value)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    cosmic::app::run::<SettingsApp>(AppSettings::default(), ())?;
    Ok(())
}
