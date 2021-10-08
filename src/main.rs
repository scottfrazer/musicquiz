use console::Term;
use rand::Rng;
use std::io::{self, Write};

mod music;

fn coming_soon() -> Page<PageState> {
    Page {
        text: String::from(
            "coming soon...\n\
             \n\
             press any key to continue",
        ),
        handler: |k, s| match k {
            _ => Action::Render(main_page()),
        },
        state: PageState::empty(),
    }
}

fn main_page() -> Page<PageState> {
    fn main_page_handler<'a>(key: console::Key, state: PageState) -> Action<PageState> {
        match key {
            console::Key::Char('1') => Action::Render(interval_quiz()),
            console::Key::Char('2') => Action::Render(coming_soon()),
            console::Key::Char('3') => Action::Render(coming_soon()),
            console::Key::Char('q') | console::Key::Char('Q') => Action::Destroy,
            _ => Action::Noop,
        }
    }

    Page {
        text: String::from(
            "=== Music Theory Quiz Game ===\n\
        \n\
        What would you like to do?\n\
        \n\
        [1] Scale Types\n\
        [2] Intervals\n\
        [3] Chords\n\
        \n\
        [q] Quit",
        ),
        handler: main_page_handler,
        state: PageState::empty(),
    }
}

fn interval_quiz() -> Page<PageState> {
    let rng = rand::thread_rng();
    let circle_of_fifths = music::circle_of_fifths();

    fn random_scale(mut rng: rand::rngs::ThreadRng, notes: Vec<music::Note>) -> music::Scale {
        let random_tonic = notes.get(rng.gen_range(0..notes.len())).unwrap();
        let random_type: music::ScaleType = rng.gen();
        music::Scale::new(random_tonic, random_type)
    }

    let scale = random_scale(rng, circle_of_fifths);

    let choices: Vec<String> = music::ScaleType::all()
        .iter()
        .enumerate()
        .map(|x| format!("[{}] {}", x.0 + 1, (*x.1).as_ref()))
        .collect();

    let correct_choice = music::ScaleType::all()
        .iter()
        .position(|t| *t == scale.scale_type())
        .unwrap();

    let text: String = format!(
        "What kind of scale is this?\n\
        \n\
        {}\n\
        \n\
        {}\n\
        \n\
        [m] Main Menu\n\
        [q] Quit",
        scale.string(),
        choices.join("\n"),
    );

    fn handler(key: console::Key, state: PageState) -> Action<PageState> {
        match key {
            console::Key::Char('m') | console::Key::Char('M') => Action::Render(main_page()),
            console::Key::Char('q') | console::Key::Char('Q') => Action::Destroy,
            console::Key::Char(c) => {
                if c >= '0' || c <= '9' {
                    let choice = c.to_digit(10).unwrap();
                    let scale = state.scale.clone();

                    if choice as usize == state.correct_choice {
                        Action::Render(Page {
                            text: format!(
                                "✅ That is correct\n\
                                \n\
                                {} is a {} scale\n\
                                \n\
                                Press any key to continue",
                                scale.string(),
                                scale.scale_type().as_ref(),
                            ),
                            handler: |k, s| -> Action<PageState> {
                                Action::Render(interval_quiz())
                            },
                            state: state.clone(),
                        })
                    } else {
                        Action::Render(Page {
                            text: format!(
                                "❌ That's not correct\n\
                                \n\
                                {} is a {} scale\n\
                                \n\
                                Press any key to continue",
                                scale.string(),
                                scale.scale_type().as_ref(),
                            ),
                            handler: |k, s| -> Action<PageState> {
                                Action::Render(interval_quiz())
                            },
                            state: state.clone(),
                        })
                    }
                } else {
                    Action::Noop
                }
            }
            _ => Action::Noop,
        }
    }

    Page {
        text,
        handler: handler,
        state: PageState {
            correct_choice: correct_choice + 1, // user enters 1-indexed values
            scale,
        },
    }
}

fn main() {
    let term = Term::buffered_stdout();
    let mut screen = QuizScreen::fullscreen(term);

    screen.init().unwrap();
    screen.run().unwrap();
    screen.destroy().unwrap();
}

enum Action<T> {
    Noop,
    Render(Page<T>),
    Destroy,
}

#[derive(Clone)]
struct PageState {
    correct_choice: usize,
    scale: music::Scale,
}

impl PageState {
    fn empty() -> PageState {
        PageState {
            correct_choice: 0,
            scale: music::Scale::new(&music::Note::parse("C"), music::ScaleType::Major),
        }
    }
}

#[derive(Clone)]
struct Page<T> {
    text: String,
    handler: fn(console::Key, T) -> Action<T>,
    state: T,
}

struct QuizScreen {
    term: console::Term,
    row: usize,
    col: usize,
    width: usize,
    height: usize,
}

impl QuizScreen {
    fn fullscreen(term: console::Term) -> QuizScreen {
        let (r, c) = term.size();
        QuizScreen {
            term,
            row: 0,
            col: 0,
            width: c as usize,
            height: r as usize,
        }
    }

    fn init(&self) -> io::Result<()> {
        self.term.hide_cursor()?;
        self.term.flush()?;
        Ok(())
    }

    fn destroy(&self) -> io::Result<()> {
        self.term.show_cursor()?;
        self.term.clear_screen()?;
        self.term.flush()?;
        Ok(())
    }

    fn run(&mut self) -> io::Result<()> {
        let mut current = main_page();
        self.render(&current.text)?;

        loop {
            match (current.handler)(self.term.read_key()?, current.state.clone()) {
                Action::Render(p) => {
                    self.render(&p.text)?;
                    current = p;
                }
                Action::Destroy => break,
                Action::Noop => (),
            }
        }

        Ok(())
    }

    fn render(&mut self, text: &str) -> io::Result<()> {
        self.term.clear_screen()?;
        self.border()?;
        self.write_page(text)?;
        self.term.flush()?;
        Ok(())
    }

    fn write_page(&mut self, page: &str) -> io::Result<()> {
        for (i, line) in page.split("\n").enumerate() {
            self.term.move_cursor_to(4, i + 2)?;
            self.term.write(line.as_bytes())?;
        }
        Ok(())
    }

    fn border(&mut self) -> io::Result<()> {
        let mut term = &self.term;

        let side = "┃".as_bytes();
        let top = "━".as_bytes();
        let tl = "┏".as_bytes();
        let bl = "┗".as_bytes();
        let tr = "┓".as_bytes();
        let br = "┛".as_bytes();

        for col in self.col..(self.width - 1) {
            term.move_cursor_to(col, 0)?;
            term.write(top)?;
            term.move_cursor_to(col, self.height - 1)?;
            term.write(top)?;
        }

        for row in self.row..(self.height - 1) {
            term.move_cursor_to(0, row)?;
            term.write(side)?;
            term.move_cursor_to(self.width - 1, row)?;
            term.write(side)?;
        }

        term.move_cursor_to(self.col, self.row)?;
        term.write(tl)?;
        term.move_cursor_to(self.col, self.height)?;
        term.write(bl)?;
        term.move_cursor_to(self.width - 1, self.row)?;
        term.write(tr)?;
        term.move_cursor_to(self.width - 1, self.height)?;
        term.write(br)?;
        Ok(())
    }
}
