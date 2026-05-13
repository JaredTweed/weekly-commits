use cosmic::app::{Core, Settings as AppSettings};
use cosmic::iced::{Alignment, Length, Limits, Size};
use cosmic::{Action, Application, ApplicationExt, Element, Task, executor, widget};
use cosmic_weekly_commits::weekly;

const REFRESH_INTERVALS: [(u64, &str); 8] = [
    (900, "15 minutes"),
    (1_800, "30 minutes"),
    (3_600, "1 hour"),
    (7_200, "2 hours"),
    (14_400, "4 hours"),
    (21_600, "6 hours"),
    (43_200, "12 hours"),
    (86_400, "24 hours"),
];

#[derive(Clone, Debug)]
enum Message {
    ServiceSelected(usize),
    ThemeSelected(usize),
    ColorModeSelected(usize),
    WeekStartSelected(usize),
    IntervalSelected(usize),
    PanelPositionSelected(usize),
    UsernameChanged(String),
    TokenChanged(String),
    InstanceChanged(String),
    PanelIndexChanged(String),
    CurrentWeekChanged(bool),
    HighlightTodayChanged(bool),
    OpenTokenUrl,
}

struct SettingsApp {
    core: Core,
    settings: weekly::Settings,
    panel_index_text: String,
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
            panel_index_text: settings.panel_index.to_string(),
            settings,
        };
        let task = app.update_title();
        (app, task)
    }

    fn update(&mut self, message: Self::Message) -> Task<Action<Self::Message>> {
        let mut should_save = true;

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
            Message::IntervalSelected(index) => {
                if let Some((seconds, _)) = REFRESH_INTERVALS.get(index) {
                    self.settings.refresh_interval = *seconds;
                }
            }
            Message::PanelPositionSelected(index) => {
                if let Some(value) = weekly::PanelPosition::ALL.get(index) {
                    self.settings.panel_position = *value;
                }
            }
            Message::UsernameChanged(value) => self.settings.github_username = value,
            Message::TokenChanged(value) => self.settings.github_token = value,
            Message::InstanceChanged(value) => self.settings.custom_instance_url = value,
            Message::PanelIndexChanged(value) => {
                self.panel_index_text = value;
                if let Ok(index) = self.panel_index_text.parse::<u32>() {
                    self.settings.panel_index = index.min(20);
                } else {
                    should_save = false;
                }
            }
            Message::CurrentWeekChanged(value) => self.settings.show_current_week_only = value,
            Message::HighlightTodayChanged(value) => self.settings.highlight_current_day = value,
            Message::OpenTokenUrl => {
                should_save = false;
                if let Err(e) = std::process::Command::new("xdg-open")
                    .arg("https://github.com/settings/personal-access-tokens/new")
                    .spawn()
                {
                    tracing::error!("failed to open token settings: {e}");
                }
            }
        }

        if should_save && let Err(e) = weekly::save_settings(&self.settings) {
            tracing::error!("failed to save settings: {e}");
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let service_labels = vec!["GitHub", "Gitea / Forgejo", "GitLab"];
        let theme_labels: Vec<&'static str> = weekly::ThemeName::ALL
            .iter()
            .map(|value| value.label())
            .collect();
        let color_mode_labels = vec!["Opacity Mode", "Grade Mode"];
        let week_labels: Vec<&'static str> = weekly::WeekdayStart::ALL
            .iter()
            .map(|value| value.label())
            .collect();
        let interval_labels: Vec<&'static str> =
            REFRESH_INTERVALS.iter().map(|(_, label)| *label).collect();
        let position_labels: Vec<&'static str> = weekly::PanelPosition::ALL
            .iter()
            .map(|value| value.label())
            .collect();

        let instance_visible = matches!(
            self.settings.service_type,
            weekly::ServiceType::Gitea | weekly::ServiceType::GitLab
        );

        let service_group = widget::settings::section()
            .title("Service Credentials")
            .add(settings_item(
                "Git Service",
                "Choose the git hosting service to track contributions from",
                dropdown(
                    service_labels,
                    selected_index(&weekly::ServiceType::ALL, self.settings.service_type),
                    Message::ServiceSelected,
                ),
            ))
            .add_maybe(instance_visible.then(|| {
                settings_item(
                    "Instance URL",
                    "",
                    widget::text_input("", &self.settings.custom_instance_url)
                        .on_input(Message::InstanceChanged)
                        .width(Length::Fixed(270.0))
                        .into(),
                )
            }))
            .add(settings_item(
                "Username",
                "",
                widget::text_input("", &self.settings.github_username)
                    .on_input(Message::UsernameChanged)
                    .width(Length::Fixed(270.0))
                    .into(),
            ))
            .add(settings_item(
                "Personal Access Token",
                "",
                widget::secure_input("", &self.settings.github_token, None, true)
                    .on_input(Message::TokenChanged)
                    .width(Length::Fixed(270.0))
                    .into(),
            ));

        let refresh_group = widget::settings::section()
            .title("Auto Update Settings")
            .add(settings_item(
                "Refresh Interval",
                "How often to check for new contributions?",
                dropdown(
                    interval_labels,
                    selected_interval(self.settings.refresh_interval),
                    Message::IntervalSelected,
                ),
            ));

        let display_group = widget::settings::section()
            .title("Display Settings")
            .add(
                widget::settings::item::builder("Highlight current day")
                    .description("Add a white border around the current day's box")
                    .toggler(
                        self.settings.highlight_current_day,
                        Message::HighlightTodayChanged,
                    ),
            )
            .add(
                widget::settings::item::builder("Show current week's commits only")
                    .description("Display commits for the current week instead of the last 7 days")
                    .toggler(
                        self.settings.show_current_week_only,
                        Message::CurrentWeekChanged,
                    ),
            )
            .add(settings_item(
                "Week starts on",
                "Select which day the week begins",
                dropdown(
                    week_labels,
                    selected_index(&weekly::WeekdayStart::ALL, self.settings.week_start_day),
                    Message::WeekStartSelected,
                ),
            ))
            .add(settings_item(
                "Color Mode",
                "Choose between opacity-based or grade-based coloring",
                dropdown(
                    color_mode_labels,
                    selected_index(&weekly::ColorMode::ALL, self.settings.color_mode),
                    Message::ColorModeSelected,
                ),
            ))
            .add(settings_item(
                "Color Theme",
                "Select a color theme for commit visualization",
                dropdown(
                    theme_labels,
                    selected_index(&weekly::ThemeName::ALL, self.settings.theme_name),
                    Message::ThemeSelected,
                ),
            ));

        let position_group = widget::settings::section()
            .title("Panel Position")
            .add(settings_item(
                "Location",
                "Which section of the panel to use",
                dropdown(
                    position_labels,
                    selected_index(&weekly::PanelPosition::ALL, self.settings.panel_position),
                    Message::PanelPositionSelected,
                ),
            ))
            .add(settings_item(
                "Index",
                "Position within the chosen section (0 is leftmost)",
                widget::text_input("", &self.panel_index_text)
                    .on_input(Message::PanelIndexChanged)
                    .width(Length::Fixed(90.0))
                    .into(),
            ));

        let info_group = widget::settings::section()
            .title(token_title(self.settings.service_type))
            .add(settings_item(
                token_row_title(self.settings.service_type),
                token_row_subtitle(self.settings.service_type),
                token_button(self.settings.service_type),
            ));

        let content = widget::Column::new()
            .spacing(24)
            .padding(24)
            .max_width(650)
            .push(service_group)
            .push(refresh_group)
            .push(display_group)
            .push(position_group)
            .push(widget::Space::new().height(Length::Fill))
            .push(info_group);

        widget::container(widget::scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .into()
    }
}

impl SettingsApp {
    fn update_title(&mut self) -> Task<Action<Message>> {
        let title = "Weekly Commits Settings".to_string();
        self.set_header_title(title.clone());
        if let Some(id) = self.core.main_window_id() {
            return self.set_window_title(title, id);
        }
        Task::none()
    }
}

fn settings_item<'a>(
    title: &'static str,
    description: &'static str,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    let item = widget::settings::item::builder(title);
    if description.is_empty() {
        item.control(control).into()
    } else {
        item.description(description).control(control).into()
    }
}

fn dropdown<'a>(
    labels: Vec<&'static str>,
    selected: Option<usize>,
    message: impl Fn(usize) -> Message + Send + Sync + 'static,
) -> Element<'a, Message> {
    widget::dropdown(labels, selected, message)
        .width(Length::Fixed(220.0))
        .into()
}

fn token_button(service_type: weekly::ServiceType) -> Element<'static, Message> {
    if service_type == weekly::ServiceType::GitHub {
        widget::button::standard("Open GitHub Token Settings")
            .on_press(Message::OpenTokenUrl)
            .into()
    } else {
        widget::text::body("").into()
    }
}

fn token_title(service_type: weekly::ServiceType) -> &'static str {
    match service_type {
        weekly::ServiceType::GitHub => "About Personal Access Tokens",
        weekly::ServiceType::Gitea => "About Gitea / Forgejo Tokens",
        weekly::ServiceType::GitLab => "About GitLab Tokens",
    }
}

fn token_row_title(service_type: weekly::ServiceType) -> &'static str {
    match service_type {
        weekly::ServiceType::GitHub => "About Personal Access Tokens",
        weekly::ServiceType::Gitea => "About Gitea / Forgejo Tokens",
        weekly::ServiceType::GitLab => "About GitLab Tokens",
    }
}

fn token_row_subtitle(service_type: weekly::ServiceType) -> &'static str {
    match service_type {
        weekly::ServiceType::GitHub => {
            "Generate a fine grained personal access token with \"All Repositories\" access."
        }
        weekly::ServiceType::Gitea => {
            "Generate an access token in your instance under Settings > Applications."
        }
        weekly::ServiceType::GitLab => {
            "Generate a Personal Access Token in your GitLab profile and enable read_api scope."
        }
    }
}

fn selected_index<T: PartialEq>(slice: &[T], value: T) -> Option<usize> {
    slice.iter().position(|item| *item == value)
}

fn selected_interval(value: u64) -> Option<usize> {
    REFRESH_INTERVALS
        .iter()
        .position(|(seconds, _)| *seconds == value)
        .or(Some(5))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let settings = AppSettings::default()
        .size(Size::new(650.0, 750.0))
        .size_limits(Limits::NONE.min_width(360.0).min_height(360.0));
    cosmic::app::run::<SettingsApp>(settings, ())?;
    Ok(())
}
