mod driver_info;
mod led_data;

use chrono::DateTime;
use driver_info::DRIVERS;
use iced::alignment;
use iced::executor;
use iced::theme::{self, Theme};
use iced::time;
use iced::widget::{button, column, container, row, text};
use iced::{
    mouse,
    widget::canvas::{self, Canvas, Frame, Path, Program},
    Alignment, Application, Color, Command, Element, Length, Point, Renderer, Settings, Size,
    Subscription,
};
use led_data::{LedCoordinate, UpdateFrame, LED_DATA};
use reqwest::Client;
use serde::Deserialize;
use std::f32;
use std::time::{Duration, Instant};
use tokio::time::sleep;

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
    led_state: bool,
    update_frames: Vec<UpdateFrame>,
    current_frame_index: usize,
    client: Client,
    driver_numbers: Vec<u32>,
}

enum State {
    Idle,
    Fetching,
    Ticking { last_tick: Instant },
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
    Reset,
    Tick(Instant),
    Blink,
    DataFetched(Result<Vec<UpdateFrame>, String>),
    FetchNextBatch,
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
                led_state: false,
                update_frames: vec![],
                current_frame_index: 0,
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
                    self.update_frames.clear();
                    self.current_frame_index = 0;
                    return Command::perform(
                        fetch_driver_data(
                            self.client.clone(),
                            self.driver_numbers.clone(),
                            0,
                            3,
                            120,
                        ),
                        Message::DataFetched,
                    );
                }
                State::Fetching => {
                    self.state = State::Idle;
                    self.led_state = false;
                }
                State::Ticking { .. } => {
                    self.state = State::Idle;
                    self.led_state = false;
                }
            },
            Message::Tick(now) => {
                if let State::Ticking { last_tick } = &mut self.state {
                    self.duration += now - *last_tick;
                    *last_tick = now;
                }
            }
            Message::Reset => {
                self.duration = Duration::default();
                self.led_state = false;
                self.current_frame_index = 0;
            }
            Message::Blink => {
                if !self.update_frames.is_empty() {
                    self.led_state = !self.led_state;
                    self.current_frame_index =
                        (self.current_frame_index + 1) % self.update_frames.len();
                }
            }
            Message::DataFetched(Ok(new_frames)) => {
                self.update_frames.extend(new_frames);
                if !self.update_frames.is_empty() {
                    self.state = State::Ticking {
                        last_tick: Instant::now(),
                    };
                    return Command::perform(
                        sleep_and_fetch_next(
                            self.client.clone(),
                            self.driver_numbers.clone(),
                            3,
                            120,
                        ),
                        Message::DataFetched,
                    );
                } else {
                    self.state = State::Idle;
                }
            }
            Message::DataFetched(Err(_)) => {
                self.state = State::Idle;
            }
            _ => {}
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = match self.state {
            State::Idle | State::Fetching => Subscription::none(),
            State::Ticking { .. } => time::every(Duration::from_millis(10)).map(Message::Tick),
        };

        let blink = match self.state {
            State::Idle | State::Fetching => Subscription::none(),
            State::Ticking { .. } => {
                time::every(Duration::from_millis(100)).map(|_| Message::Blink)
            }
        };

        Subscription::batch(vec![tick, blink])
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
            button(text(label).horizontal_alignment(alignment::Horizontal::Center))
                .padding(10)
                .width(80)
        };

        let toggle_button = {
            let label = match self.state {
                State::Idle | State::Fetching => "Start",
                State::Ticking { .. } => "Stop",
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
            .spacing(20),
        )
        .align_x(alignment::Horizontal::Right)
        .align_y(alignment::Vertical::Bottom)
        .width(Length::FillPortion(1));

        let bottom_row = row![duration_container, buttons_container].width(Length::Fill);

        let canvas = Canvas::new(Graph {
            data: LED_DATA.to_vec(),
            led_state: self.led_state,
            update_frames: self.update_frames.clone(),
            current_frame_index: self.current_frame_index,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        container(column![canvas, bottom_row].spacing(20))
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
    led_state: bool,
    update_frames: Vec<UpdateFrame>,
    current_frame_index: usize,
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
        if !self.update_frames.is_empty() {
            let frame_data = &self.update_frames[self.current_frame_index];

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
    start_index: usize,
    drivers_per_batch: usize,
    entries_per_driver: usize,
) -> Result<Vec<UpdateFrame>, String> {
    let session_key = "9149";
    let start_time: &str = "2023-08-27T12:58:56.200";
    let end_time: &str = "2023-08-27T13:20:54.300";

    let mut all_data: Vec<LocationData> = Vec::new();

    for chunk_start in (start_index..driver_numbers.len()).step_by(drivers_per_batch) {
        for driver_number in &driver_numbers
            [chunk_start..chunk_start + drivers_per_batch.min(driver_numbers.len() - chunk_start)]
        {
            let mut fetched_entries = 0;

            while fetched_entries < entries_per_driver {
                let url = format!(
                    "https://api.openf1.org/v1/location?session_key={}&driver_number={}&date>{}&date<{}",
                    session_key, driver_number, start_time, end_time,
                );
                eprintln!("url: {}", url);
                let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
                if resp.status().is_success() {
                    let data: Vec<LocationData> = resp.json().await.map_err(|e| e.to_string())?;
                    let valid_data: Vec<LocationData> = data
                        .into_iter()
                        .filter(|d| d.x != 0.0 && d.y != 0.0)
                        .collect();
                    fetched_entries += valid_data.len();
                    eprintln!(
                        "Fetched {} entries for driver number {}",
                        valid_data.len(),
                        driver_number
                    );
                    all_data.extend(valid_data);
                } else {
                    eprintln!(
                        "Failed to fetch data for driver {}: HTTP {}",
                        driver_number,
                        resp.status()
                    );
                    break;
                }
            }
        }
    }

    // Sort the data by the date field
    all_data.sort_by_key(|d| d.date.clone());

    let mut update_frames = Vec::new();
    let mut current_frame: Option<UpdateFrame> = None;

    for data in all_data {
        let timestamp = DateTime::parse_from_rfc3339(&data.date)
            .map_err(|e| e.to_string())?
            .timestamp_millis() as u64;
        let x = data.x;
        let y = data.y;
        let driver_number = data.driver_number;

        let driver = match DRIVERS.iter().find(|d| d.number == driver_number) {
            Some(d) => d,
            None => {
                eprintln!("Driver not found for number: {}", driver_number);
                continue;
            }
        };

        let color = driver.color;

        let nearest_led = LED_DATA
            .iter()
            .min_by(|a, b| {
                let dist_a = ((a.x_led - x).powi(2) + (a.y_led - y).powi(2)).sqrt();
                let dist_b = ((b.x_led - x).powi(2) + (b.y_led - y).powi(2)).sqrt();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .unwrap();

        if let Some(frame) = &mut current_frame {
            if frame.timestamp == timestamp {
                frame.set_led_state(nearest_led.led_number, color);
            } else {
                update_frames.push(frame.clone());
                current_frame = Some(UpdateFrame::new(timestamp));
                current_frame
                    .as_mut()
                    .unwrap()
                    .set_led_state(nearest_led.led_number, color);
            }
        } else {
            current_frame = Some(UpdateFrame::new(timestamp));
            current_frame
                .as_mut()
                .unwrap()
                .set_led_state(nearest_led.led_number, color);
        }
    }

    if let Some(frame) = current_frame {
        update_frames.push(frame);
    }

    Ok(update_frames)
}

async fn sleep_and_fetch_next(
    client: Client,
    driver_numbers: Vec<u32>,
    drivers_per_batch: usize,
    entries_per_driver: usize,
) -> Result<Vec<UpdateFrame>, String> {
    sleep(Duration::from_millis(334)).await;
    fetch_driver_data(
        client,
        driver_numbers,
        0,
        drivers_per_batch,
        entries_per_driver,
    )
    .await
}
