mod led_data;

use iced::alignment;
use iced::executor;
use iced::keyboard;
use iced::theme::{self, Theme};
use iced::time;
use iced::widget::{button, container, row, text, column};
use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
    widget::canvas::{self, Canvas, Path, Frame, Program}, Color, Point, Size, mouse, Renderer
};

use std::time::{Duration, Instant};
use led_data::{LedCoordinate, LED_DATA};

pub fn main() -> iced::Result {
    Stopwatch::run(Settings::default())
}

struct Stopwatch {
    duration: Duration,
    state: State,
    blink_state: bool,
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

impl Application for Stopwatch {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Stopwatch, Command<Message>) {
        (
            Stopwatch {
                duration: Duration::default(),
                state: State::Idle,
                blink_state: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Stopwatch - Iced")
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
            }
            Message::Blink => {
                self.blink_state = !self.blink_state;
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
                time::every(Duration::from_millis(500)).map(|_| Message::Blink)
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

        // Draw the blinking rectangle
        let color = if self.blink_state { Color::from_rgb(0.0, 0.0, 1.0) } else { Color::from_rgb(1.0, 0.0, 0.0) };
        let blinking_rect = Path::rectangle(
            Point::new((0.0 - min_x) * scale_x + padding, bounds.height - (0.0 - min_y) * scale_y - padding - 10.0),
            Size::new(10.0, 10.0),
        );
        frame.fill(&blinking_rect, color);

        for led in &self.data {
            let x = (led.x_led - min_x) * scale_x + padding;
            let y = bounds.height - (led.y_led - min_y) * scale_y - padding;

            let point = Path::rectangle(Point::new(x, y), Size::new(5.0, 5.0));
            frame.fill(&point, Color::from_rgb(0.0, 1.0, 0.0));
        }

        vec![frame.into_geometry()]
    }
}
