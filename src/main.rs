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

use std::time::{Duration, Instant};
use led_data::{LedCoordinate, LED_DATA, UpdateFrame};
use driver_info::DRIVERS;
use std::fs::File;
use std::io::BufReader;
use csv::Reader;

pub fn main() -> iced::Result {
    Race::run(Settings::default())
}

struct Race {
    duration: Duration,
    state: State,
    blink_state: bool,
    update_frames: Vec<UpdateFrame>,
    current_frame_index: usize,
}

enum State {
    Idle,
    Ticking { last_tick: Instant },
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
    Reset,
    Tick(Instant),
    Blink,
}

impl Application for Race {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Race, Command<Message>) {
        let update_frames = load_update_frames("processed_100k.csv");

        (
            Race {
                duration: Duration::default(),
                state: State::Idle,
                blink_state: false,
                update_frames,
                current_frame_index: 0,
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
                    self.state = State::Ticking {
                        last_tick: Instant::now(),
                    };
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
                self.blink_state = !self.blink_state;
                self.current_frame_index = (self.current_frame_index + 1) % self.update_frames.len();
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = match self.state {
            State::Idle => Subscription::none(),
            State::Ticking { .. } => {
                time::every(Duration::from_millis(10)).map(Message::Tick)
            }
        };

        let blink = match self.state {
            State::Idle => Subscription::none(),
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
                State::Idle => "Start",
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

        vec![frame.into_geometry()]
    }
}

fn load_update_frames(file_path: &str) -> Vec<UpdateFrame> {
    let file = File::open(file_path).expect("Unable to open file");
    let mut rdr = Reader::from_reader(BufReader::new(file));

    let mut update_frames = Vec::new();
    let mut current_frame: Option<UpdateFrame> = None;

    for result in rdr.records() {
        match result {
            Ok(record) => {
                let timestamp: u64 = match record.get(2).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.timestamp_millis() as u64)) {
                    Some(t) => t,
                    None => {
                        eprintln!("Invalid timestamp: {:?}", record.get(2));
                        continue;
                    }
                };

                let led_number: u32 = match record.get(4).and_then(|s| s.parse().ok()) {
                    Some(n) => n,
                    None => {
                        eprintln!("Invalid LED number: {:?}", record.get(4));
                        continue;
                    }
                };

                let driver_number: u32 = match record.get(3).and_then(|s| s.parse().ok()) {
                    Some(n) => n,
                    None => {
                        eprintln!("Invalid driver number: {:?}", record.get(3));
                        continue;
                    }
                };

                let driver = match DRIVERS.iter().find(|d| d.number == driver_number) {
                    Some(d) => d,
                    None => {
                        eprintln!("Driver not found for number: {}", driver_number);
                        continue;
                    }
                };

                let color = driver.color;

                if let Some(frame) = &mut current_frame {
                    if frame.timestamp == timestamp {
                        frame.set_led_state(led_number, color);
                    } else {
                        update_frames.push(frame.clone());
                        current_frame = Some(UpdateFrame::new(timestamp));
                        current_frame.as_mut().unwrap().set_led_state(led_number, color);
                    }
                } else {
                    current_frame = Some(UpdateFrame::new(timestamp));
                    current_frame.as_mut().unwrap().set_led_state(led_number, color);
                }
            }
            Err(e) => {
                eprintln!("Error reading record: {}", e);
            }
        }
    }

    if let Some(frame) = current_frame {
        update_frames.push(frame);
    }

    update_frames
}

