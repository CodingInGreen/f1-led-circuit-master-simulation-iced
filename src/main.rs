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
use chrono::DateTime;

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
    blink_state: bool,
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
    FetchNextDriver,
    DataFetched(Result<Vec<UpdateFrame>, String>),
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
                blink_state: false,
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
                    return Command::perform(fetch_driver_data(self.client.clone(), self.driver_numbers.clone(), 0), Message::DataFetched);
                }
                State::Fetching => {
                    self.state = State::Idle;
                    self.blink_state = false;
                }
                State::Ticking { .. } => {
                    self.state = State::Idle;
                    self.blink_state = false;
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
                self.blink_state = false;
                self.current_frame_index = 0;
            }
            Message::Blink => {
                if !self.update_frames.is_empty() {
                    self.blink_state = !self.blink_state;
                    self.current_frame_index = (self.current_frame_index + 1) % self.update_frames.len();
                }
            }
            Message::DataFetched(Ok(new_frames)) => {
                self.update_frames.extend(new_frames);
                if !self.update_frames.is_empty() {
                    self.state = State::Ticking {
                        last_tick: Instant::now(),
                    };
                } else {
                    self.state = State::Idle;
                }
                if (self.update_frames.len() / 20) < self.driver_numbers.len() {
                    return Command::perform(fetch_driver_data(self.client.clone(), self.driver_numbers.clone(), self.update_frames.len() / 20), Message::DataFetched);
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
            State::Ticking { .. } => {
                time::every(Duration::from_millis(10)).map(Message::Tick)
            }
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
                State::Ticking { .. } => "Stop",
            };

            button(label).on_press(Message::Toggle)
        };

        let reset_button = button("Reset")
            .style(theme::Button::Destructive)
            .on_press(Message::Reset);

        let content = row![
            container(duration).padding(10),
            container(toggle_button).padding(10),
            container(reset_button).padding(10)
        ]
        .align_items(Alignment::Center)
        .spacing(20);

        let canvas = Canvas::new(Graph {
            data: LED_DATA.to_vec(),
            blink_state: self.blink_state,
            update_frames: self.update_frames.clone(),
            current_frame_index: self.current_frame_index,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        container(column![canvas, content])
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .padding(20)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

struct Graph {
    data: Vec<LedCoordinate>,
    blink_state: bool,
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
                    .unwrap_or(Color::from_rgb(0.0, 1.0, 0.0));

                let point = Path::rectangle(Point::new(x, y), Size::new(10.0, 10.0));
                frame.fill(&point, color);
            }
        }

        vec![frame.into_geometry()]
    }
}

async fn fetch_driver_data(client: Client, driver_numbers: Vec<u32>, start_index: usize) -> Result<Vec<UpdateFrame>, String> {
    let session_key = "9149";
    let start_time: &str = "2023-08-27T12:58:56.200";
    let end_time: &str = "2023-08-27T13:20:54.300";

    let mut all_data: Vec<LocationData> = Vec::new();

    for driver_number in &driver_numbers[start_index..start_index + 1] {
        let url = format!(
            "https://api.openf1.org/v1/location?session_key={}&driver_number={}&date>{}&date<{}",
            session_key, driver_number, start_time, end_time,
        );
        eprintln!("url: {}", url);
        let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            let data: Vec<LocationData> = resp.json().await.map_err(|e| e.to_string())?;
            all_data.extend(data.into_iter().filter(|d| d.x != 0.0 && d.y != 0.0));
        } else {
            eprintln!(
                "Failed to fetch data for driver {}: HTTP {}",
                driver_number,
                resp.status()
            );
        }
    }

    // Sort the data by the date field
    all_data.sort_by_key(|d| d.date.clone());

    let mut update_frames = Vec::new();
    let mut current_frame: Option<UpdateFrame> = None;

    for data in all_data {
        let timestamp = DateTime::parse_from_rfc3339(&data.date).map_err(|e| e.to_string())?.timestamp_millis() as u64;
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

        let nearest_led = LED_DATA.iter()
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
                current_frame.as_mut().unwrap().set_led_state(nearest_led.led_number, color);
            }
        } else {
            current_frame = Some(UpdateFrame::new(timestamp));
            current_frame.as_mut().unwrap().set_led_state(nearest_led.led_number, color);
        }
    }

    if let Some(frame) = current_frame {
        update_frames.push(frame);
    }

    Ok(update_frames)
}