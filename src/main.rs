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

#[derive(Debug, Clone)]
struct LedCoordinate {
    x_led: f32,
    y_led: f32,
    led_number: u32,
}

const LED_DATA: &[LedCoordinate] = &[
    LedCoordinate { x_led: 6413.0, y_led: 33.0, led_number: 1 },
    LedCoordinate { x_led: 6007.0, y_led: 197.0, led_number: 2 },
    LedCoordinate { x_led: 5652.0, y_led: 444.0, led_number: 3 },
    LedCoordinate { x_led: 5431.0, y_led: 822.0, led_number: 4 },
    LedCoordinate { x_led: 5727.0, y_led: 1143.0, led_number: 5 },
    LedCoordinate { x_led: 6141.0, y_led: 1268.0, led_number: 6 },
    LedCoordinate { x_led: 6567.0, y_led: 1355.0, led_number: 7 },
    LedCoordinate { x_led: 6975.0, y_led: 1482.0, led_number: 8 },
    LedCoordinate { x_led: 7328.0, y_led: 1738.0, led_number: 9 },
    LedCoordinate { x_led: 7369.0, y_led: 2173.0, led_number: 10 },
    LedCoordinate { x_led: 7024.0, y_led: 2448.0, led_number: 11 },
    LedCoordinate { x_led: 6592.0, y_led: 2505.0, led_number: 12 },
    LedCoordinate { x_led: 6159.0, y_led: 2530.0, led_number: 13 },
    LedCoordinate { x_led: 5725.0, y_led: 2525.0, led_number: 14 },
    LedCoordinate { x_led: 5288.0, y_led: 2489.0, led_number: 15 },
    LedCoordinate { x_led: 4857.0, y_led: 2434.0, led_number: 16 },
    LedCoordinate { x_led: 4429.0, y_led: 2356.0, led_number: 17 },
    LedCoordinate { x_led: 4004.0, y_led: 2249.0, led_number: 18 },
    LedCoordinate { x_led: 3592.0, y_led: 2122.0, led_number: 19 },
    LedCoordinate { x_led: 3181.0, y_led: 1977.0, led_number: 20 },
    LedCoordinate { x_led: 2779.0, y_led: 1812.0, led_number: 21 },
    LedCoordinate { x_led: 2387.0, y_led: 1624.0, led_number: 22 },
    LedCoordinate { x_led: 1988.0, y_led: 1453.0, led_number: 23 },
    LedCoordinate { x_led: 1703.0, y_led: 1779.0, led_number: 24 },
    LedCoordinate { x_led: 1271.0, y_led: 1738.0, led_number: 25 },
    LedCoordinate { x_led: 1189.0, y_led: 1314.0, led_number: 26 },
    LedCoordinate { x_led: 1257.0, y_led: 884.0, led_number: 27 },
    LedCoordinate { x_led: 1333.0, y_led: 454.0, led_number: 28 },
    LedCoordinate { x_led: 1409.0, y_led: 25.0, led_number: 29 },
    LedCoordinate { x_led: 1485.0, y_led: -405.0, led_number: 30 },
    LedCoordinate { x_led: 1558.0, y_led: -835.0, led_number: 31 },
    LedCoordinate { x_led: 1537.0, y_led: -1267.0, led_number: 32 },
    LedCoordinate { x_led: 1208.0, y_led: -1555.0, led_number: 33 },
    LedCoordinate { x_led: 779.0, y_led: -1606.0, led_number: 34 },
    LedCoordinate { x_led: 344.0, y_led: -1604.0, led_number: 35 },
    LedCoordinate { x_led: -88.0, y_led: -1539.0, led_number: 36 },
    LedCoordinate { x_led: -482.0, y_led: -1346.0, led_number: 37 },
    LedCoordinate { x_led: -785.0, y_led: -1038.0, led_number: 38 },
    LedCoordinate { x_led: -966.0, y_led: -644.0, led_number: 39 },
    LedCoordinate { x_led: -1015.0, y_led: -206.0, led_number: 40 },
    LedCoordinate { x_led: -923.0, y_led: 231.0, led_number: 41 },
    LedCoordinate { x_led: -762.0, y_led: 650.0, led_number: 42 },
    LedCoordinate { x_led: -591.0, y_led: 1078.0, led_number: 43 },
    LedCoordinate { x_led: -423.0, y_led: 1497.0, led_number: 44 },
    LedCoordinate { x_led: -254.0, y_led: 1915.0, led_number: 45 },
    LedCoordinate { x_led: -86.0, y_led: 2329.0, led_number: 46 },
    LedCoordinate { x_led: 83.0, y_led: 2744.0, led_number: 47 },
    LedCoordinate { x_led: 251.0, y_led: 3158.0, led_number: 48 },
    LedCoordinate { x_led: 416.0, y_led: 3574.0, led_number: 49 },
    LedCoordinate { x_led: 588.0, y_led: 3990.0, led_number: 50 },
    LedCoordinate { x_led: 755.0, y_led: 4396.0, led_number: 51 },
    LedCoordinate { x_led: 920.0, y_led: 4804.0, led_number: 52 },
    LedCoordinate { x_led: 1086.0, y_led: 5212.0, led_number: 53 },
    LedCoordinate { x_led: 1250.0, y_led: 5615.0, led_number: 54 },
    LedCoordinate { x_led: 1418.0, y_led: 6017.0, led_number: 55 },
    LedCoordinate { x_led: 1583.0, y_led: 6419.0, led_number: 56 },
    LedCoordinate { x_led: 1909.0, y_led: 6702.0, led_number: 57 },
    LedCoordinate { x_led: 2306.0, y_led: 6512.0, led_number: 58 },
    LedCoordinate { x_led: 2319.0, y_led: 6071.0, led_number: 59 },
    LedCoordinate { x_led: 2152.0, y_led: 5660.0, led_number: 60 },
    LedCoordinate { x_led: 1988.0, y_led: 5255.0, led_number: 61 },
    LedCoordinate { x_led: 1853.0, y_led: 4836.0, led_number: 62 },
    LedCoordinate { x_led: 1784.0, y_led: 4407.0, led_number: 63 },
    LedCoordinate { x_led: 1779.0, y_led: 3971.0, led_number: 64 },
    LedCoordinate { x_led: 1605.0, y_led: 3569.0, led_number: 65 },
    LedCoordinate { x_led: 1211.0, y_led: 3375.0, led_number: 66 },
    LedCoordinate { x_led: 811.0, y_led: 3188.0, led_number: 67 },
    LedCoordinate { x_led: 710.0, y_led: 2755.0, led_number: 68 },
    LedCoordinate { x_led: 1116.0, y_led: 2595.0, led_number: 69 },
    LedCoordinate { x_led: 1529.0, y_led: 2717.0, led_number: 70 },
    LedCoordinate { x_led: 1947.0, y_led: 2848.0, led_number: 71 },
    LedCoordinate { x_led: 2371.0, y_led: 2946.0, led_number: 72 },
    LedCoordinate { x_led: 2806.0, y_led: 2989.0, led_number: 73 },
    LedCoordinate { x_led: 3239.0, y_led: 2946.0, led_number: 74 },
    LedCoordinate { x_led: 3665.0, y_led: 2864.0, led_number: 75 },
    LedCoordinate { x_led: 4092.0, y_led: 2791.0, led_number: 76 },
    LedCoordinate { x_led: 4523.0, y_led: 2772.0, led_number: 77 },
    LedCoordinate { x_led: 4945.0, y_led: 2886.0, led_number: 78 },
    LedCoordinate { x_led: 5331.0, y_led: 3087.0, led_number: 79 },
    LedCoordinate { x_led: 5703.0, y_led: 3315.0, led_number: 80 },
    LedCoordinate { x_led: 6105.0, y_led: 3484.0, led_number: 81 },
    LedCoordinate { x_led: 6538.0, y_led: 3545.0, led_number: 82 },
    LedCoordinate { x_led: 6969.0, y_led: 3536.0, led_number: 83 },
    LedCoordinate { x_led: 7402.0, y_led: 3511.0, led_number: 84 },
    LedCoordinate { x_led: 7831.0, y_led: 3476.0, led_number: 85 },
    LedCoordinate { x_led: 8241.0, y_led: 3335.0, led_number: 86 },
    LedCoordinate { x_led: 8549.0, y_led: 3025.0, led_number: 87 },
    LedCoordinate { x_led: 8703.0, y_led: 2612.0, led_number: 88 },
    LedCoordinate { x_led: 8662.0, y_led: 2173.0, led_number: 89 },
    LedCoordinate { x_led: 8451.0, y_led: 1785.0, led_number: 90 },
    LedCoordinate { x_led: 8203.0, y_led: 1426.0, led_number: 91 },
    LedCoordinate { x_led: 7973.0, y_led: 1053.0, led_number: 92 },
    LedCoordinate { x_led: 7777.0, y_led: 664.0, led_number: 93 },
    LedCoordinate { x_led: 7581.0, y_led: 275.0, led_number: 94 },
    LedCoordinate { x_led: 7274.0, y_led: -35.0, led_number: 95 },
    LedCoordinate { x_led: 6839.0, y_led: -46.0, led_number: 96 },
];

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

        let scale_x = bounds.width / width;
        let scale_y = bounds.height / height;

        // Draw the blinking rectangle
        let color = if self.blink_state { Color::from_rgb(0.0, 0.0, 1.0) } else { Color::from_rgb(1.0, 0.0, 0.0) };
        let blinking_rect = Path::rectangle(
            Point::new((0.0 - min_x) * scale_x, bounds.height - (0.0 - min_y) * scale_y - 10.0),
            Size::new(10.0, 10.0),
        );
        frame.fill(&blinking_rect, color);

        for led in &self.data {
            let x = (led.x_led - min_x) * scale_x;
            let y = bounds.height - (led.y_led - min_y) * scale_y;

            let point = Path::rectangle(Point::new(x, y), Size::new(5.0, 5.0));
            frame.fill(&point, Color::from_rgb(0.0, 1.0, 0.0));
        }

        vec![frame.into_geometry()]
    }
}