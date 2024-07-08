mod led_data;
mod driver_info;

use iced::alignment;
use iced::executor;
use iced::theme::{self, Theme};
use iced::time;
use iced::widget::{button, container, row, text, column};
use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
    widget::canvas::{self, Canvas, Path, Frame, Program}, Color, Point, Size, mouse, Renderer
};
use reqwest::Client;
use serde::Deserialize;
use std::time::{Duration, Instant};
use led_data::{LedCoordinate, LED_DATA, UpdateFrame};
use driver_info::DRIVERS;
use std::f32;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
struct LocationData {
    x: f32,
    y: f32,
    date: String,
    driver_number: u32,
}

pub fn main() -> iced::Result {
    Race::run(Settings::default())
}

struct Race {
    duration: Duration,
    state: State,
    update_frame: Option<UpdateFrame>,
    client: Client,
    driver_numbers: Vec<u32>,
}

enum State {
    Idle,
    Fetching,
    Displaying,
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
    Reset,
    Tick(Instant),
    DataFetched(Result<UpdateFrame, String>),
}

impl Application for Race {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Race, Command<Message>) {
        (
            Race {
                duration: Duration::default(),
                state: State::Idle,
                update_frame: None,
                client: Client::new(),
                driver_numbers: vec![
                    1, 2, 4, 10, 11, 14, 16, 18, 20, 22, 23, 24, 27, 31, 40, 44, 55, 63, 77, 81,
                ],
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("F1-LED-CIRCUIT")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Toggle => match self.state {
                State::Idle => {
                    self.state = State::Fetching;
                    self.update_frame = None;
                    return Command::perform(
                        fetch_driver_data(
                            self.client.clone(),
                            self.driver_numbers.clone(),
                            3,
                            20,
                        ), 
                        Message::DataFetched
                    );
                }
                State::Fetching => {
                    self.state = State::Idle;
                }
                State::Displaying => {
                    self.state = State::Idle;
                }
            },
            Message::Tick(now) => {
                if let State::Displaying = &mut self.state {
                    self.duration += now - Instant::now();
                }
            }
            Message::Reset => {
                self.duration = Duration::default();
                self.update_frame = None;
            }
            Message::DataFetched(Ok(new_frame)) => {
                self.update_frame = Some(new_frame);
                if self.update_frame.is_some() {
                    self.state = State::Displaying;
                } else {
                    self.state = State::Idle;
                }
            }
            Message::DataFetched(Err(_)) => {
                self.state = State::Idle;
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Idle | State::Fetching => Subscription::none(),
            State::Displaying => time::every(Duration::from_millis(1000)).map(Message::Tick),
        }
    }

    fn view(&self) -> Element<Message> {
        if let State::Fetching = self.state {
            return container(
                text("DOWNLOADING DATA...")
                    .size(50)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into();
        }

        const MINUTE: u64 = 60;
        const HOUR: u64 = 60 * MINUTE;

        let seconds = self.duration.as_secs();

        let duration = text(format!(
            "{:0>2}:{:0>2}:{:0>2}.{:0>2}",
            seconds / HOUR,
            (seconds % HOUR) / MINUTE,
            seconds % MINUTE,
            self.duration.subsec_millis() / 10,
        ))
        .size(40);

        let button = |label| {
            button(
                text(label).horizontal_alignment(alignment::Horizontal::Center),
            )
            .padding(10)
            .width(80)
        };

        let toggle_button = {
            let label = match self.state {
                State::Idle | State::Fetching => "Start",
                State::Displaying => "Stop",
            };

            button(label).on_press(Message::Toggle)
        };

        let reset_button = button("Reset")
            .style(theme::Button::Destructive)
            .on_press(Message::Reset);

        let duration_container = container(duration)
            .padding(10)
            .align_x(alignment::Horizontal::Left)
            .align_y(alignment::Vertical::Bottom)
            .width(Length::FillPortion(1));

        let buttons_container = container(
            row![
                container(toggle_button).padding(10),
                container(reset_button).padding(10)
            ]
            .align_items(Alignment::Center)
            .spacing(20)
        )
        .align_x(alignment::Horizontal::Right)
        .align_y(alignment::Vertical::Bottom)
        .width(Length::FillPortion(1));

        let bottom_row = row![
            duration_container,
            buttons_container
        ]
        .width(Length::Fill);

        let canvas = Canvas::new(Graph {
            data: LED_DATA.to_vec(),
            update_frame: self.update_frame.clone(),
        })
        .width(Length::Fill)
        .height(Length::Fill);

        container(
            column![
                canvas,
                bottom_row
            ]
            .spacing(20)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }
}

struct Graph {
    data: Vec<LedCoordinate>,
    update_frame: Option<UpdateFrame>,
}

impl<Message> Program<Message> for Graph {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        _renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(_renderer, bounds.size());

        let (min_x, max_x, min_y, max_y) = self.data.iter().fold(
            (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
            |(min_x, max_x, min_y, max_y), led| {
                (
                    min_x.min(led.x_led),
                    max_x.max(led.x_led),
                    min_y.min(led.y_led),
                    max_y.max(led.y_led),
                )
            },
        );

        let width = max_x - min_x;
        let height = max_y - min_y;

        // Apply padding
        let padding = 50.0;
        let scale_x = (bounds.width - 2.0 * padding) / width;
        let scale_y = (bounds.height - 2.0 * padding) / height;

        // Draw the LED rectangles
        if let Some(frame_data) = &self.update_frame {
            for led in &self.data {
                let x = (led.x_led - min_x) * scale_x + padding;
                let y = bounds.height - (led.y_led - min_y) * scale_y - padding;

                let color = frame_data
                    .led_states
                    .iter()
                    .find(|(num, _)| *num == led.led_number)
                    .map(|(_, col)| Color::from_rgb8(col.0, col.1, col.2))
                    .unwrap_or(Color::from_rgb(0.0, 0.0, 0.0));

                let point = Path::rectangle(Point::new(x, y), Size::new(10.0, 10.0));
                frame.fill(&point, color);
            }
        }

        vec![frame.into_geometry()]
    }
}

async fn fetch_driver_data(
    client: Client,
    driver_numbers: Vec<u32>,
    drivers_per_batch: usize,
    max_calls: usize,
) -> Result<UpdateFrame, String> {
    let session_key = "9149";

    let mut update_frame = UpdateFrame::new(0);
    let mut call_count = 0;

    for chunk_start in (0..driver_numbers.len()).step_by(drivers_per_batch) {
        for driver_number in &driver_numbers[chunk_start..chunk_start + drivers_per_batch.min(driver_numbers.len() - chunk_start)] {
            if call_count >= max_calls {
                break;
            }
            let url = format!(
                "https://api.openf1.org/v1/location?session_key={}&driver_number={}",
                session_key, driver_number,
            );
            eprintln!("url: {}", url);
            let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                let data: Vec<LocationData> = resp.json().await.map_err(|e| e.to_string())?;
                if let Some(location) = data.into_iter().filter(|d| d.x != 0.0 && d.y != 0.0).next() {
                    eprintln!("Fetched and using data: {:?}", location); // Print debug statement

                    let driver = match DRIVERS.iter().find(|d| d.number == location.driver_number) {
                        Some(d) => d,
                        None => {
                            eprintln!("Driver not found for number: {}", location.driver_number);
                            continue;
                        }
                    };

                    let color = driver.color;

                    let nearest_led = LED_DATA.iter()
                        .min_by(|a, b| {
                            let dist_a = ((a.x_led - location.x).powi(2) + (a.y_led - location.y).powi(2)).sqrt();
                            let dist_b = ((b.x_led - location.x).powi(2) + (b.y_led - location.y).powi(2)).sqrt();
                            dist_a.partial_cmp(&dist_b).unwrap()
                        })
                        .unwrap();

                    update_frame.set_led_state(nearest_led.led_number, color);
                    call_count += 1;

                    // Break out of the loop after processing one valid LocationData
                    break;
                } else {
                    eprintln!("No valid data found for driver {}", driver_number);
                }
            } else {
                eprintln!(
                    "Failed to fetch data for driver {}: HTTP {}",
                    driver_number,
                    resp.status()
                );
            }
        }
        if call_count >= max_calls {
            break;
        }
    }

    Ok(update_frame)
}
