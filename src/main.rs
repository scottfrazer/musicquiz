use console::Term;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use regex::Regex;
use std::convert::AsRef;
use std::env;
use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;
use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::EnumIter;

const ALPHABET: [char; 7] = ['A', 'B', 'C', 'D', 'E', 'F', 'G'];

enum Bias {
    Flat,
    Sharp,
}

enum Interval {
    Unison,
    MinorSecond,
    MajorSecond,
    MinorThird,
    MajorThird,
    PerfectFourth,
    Tritone,
    PerfectFifth,
    MinorSixth,
    MajorSixth,
    MinorSeventh,
    MajorSeventh,
}

#[derive(Debug, EnumIter, AsRefStr, PartialEq)]
enum ScaleType {
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
}

impl ScaleType {
    fn interval_pattern(&self) -> &[i8] {
        match self {
            &Self::Major => &[0, 2, 2, 1, 2, 2, 2, 1],
            &Self::Minor => &[0, 2, 1, 2, 2, 1, 2, 2],
            &Self::Dorian => &[0, 2, 1, 2, 2, 2, 1, 2],
            &Self::Phrygian => &[0, 1, 2, 2, 2, 1, 2, 2],
            &Self::Lydian => &[0, 2, 2, 2, 1, 2, 2, 1],
            &Self::Mixolydian => &[0, 2, 2, 1, 2, 2, 1, 2],
            &Self::Locrian => &[0, 1, 2, 2, 1, 2, 2, 2],
        }
    }

    fn from(s: &str) -> ScaleType {
        match &s.to_lowercase()[..] {
            "major" => ScaleType::Major,
            "minor" => ScaleType::Minor,
            "dorian" => ScaleType::Dorian,
            "phrygian" => ScaleType::Phrygian,
            "lydian" => ScaleType::Lydian,
            "mixolydian" => ScaleType::Mixolydian,
            "locrian" => ScaleType::Locrian,
            _ => ScaleType::Major, // todo
        }
    }
}

impl Distribution<ScaleType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ScaleType {
        match rng.gen_range(0..7) {
            0 => ScaleType::Major,
            1 => ScaleType::Minor,
            2 => ScaleType::Dorian,
            3 => ScaleType::Phrygian,
            4 => ScaleType::Lydian,
            5 => ScaleType::Mixolydian,
            6 => ScaleType::Locrian,
            _ => ScaleType::Major, // can't happen
        }
    }
}

fn base(letter: char) -> Vec<char> {
    let mut scale = Vec::with_capacity(7);
    for (i, letter2) in ALPHABET.iter().enumerate() {
        if letter != *letter2 {
            continue;
        }
        for j in i..i + ALPHABET.len() {
            scale.push(ALPHABET[j % ALPHABET.len()])
        }
    }
    scale
}

fn pitch_class(letter: char) -> i8 {
    match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => 127, // TODO
    }
}

fn parse_adjustment(suffix: &str) -> i8 {
    match suffix {
        "##" | "♯♯" | "𝄪" => 2,
        "#" | "♯" => 1,
        "b" | "♭" => -1,
        "bb" | "♭♭" | "𝄫" => -2,
        _ => 0, // TODO
    }
}

fn pitch_class_to_letter(pc: i8, bias: Bias) -> char {
    match bias {
        Bias::Sharp => match pc % 12 {
            0 | 1 => 'C',
            2 | 3 => 'D',
            4 => 'E',
            5 | 6 => 'F',
            7 | 8 => 'G',
            9 | 10 => 'A',
            11 => 'B',
            _ => ' ',
        },
        Bias::Flat => match pc % 12 {
            0 => 'C',
            1 | 2 => 'D',
            3 | 4 => 'E',
            5 => 'F',
            6 | 7 => 'G',
            8 | 9 => 'A',
            10 | 11 => 'B',
            _ => ' ',
        },
    }
}

struct Chord {
    notes: Vec<Note>,
    name: String,
}

impl Chord {
    fn parse(s: &str) -> Chord {
        Chord {
            notes: Vec::new(),
            name: String::from("foobar"),
        }
    }
}

struct Scale {
    notes: Vec<Note>,
    scale_type: ScaleType,
}

impl Scale {
    fn new(tonic: &Note, scale_type: ScaleType) -> Scale {
        let mut notes = Vec::new();
        for (j, letter) in base(tonic.spelling).iter().enumerate() {
            let half_steps: i8 = scale_type.interval_pattern()[..j + 1].iter().sum();
            let degree = tonic.add(half_steps, *letter);
            notes.push(degree)
        }
        Scale { notes, scale_type }
    }

    fn string(&self) -> String {
        let strings: Vec<String> = self.notes.iter().map(|x| x.string()).collect();
        let scale = strings.join(" ");
        format!(
            "{} {}: {}",
            self.tonic().string(),
            self.scale_type.as_ref(),
            scale,
        )
    }

    fn tonic(&self) -> &Note {
        self.notes.get(0).unwrap()
    }
}

struct Pitch {
    pitch_class: i8,
    octave: i8,
}

impl Pitch {
    fn parse(s: String) -> Pitch {
        let re = Regex::new(r"([ABCDEFG])([#♯𝄪b♭𝄫]*)([0-9]+)").unwrap();
        let mut pc: i8 = 0;
        let mut o: i8 = 0;
        for caps in re.captures_iter(&s) {
            let letter = caps.get(1).unwrap().as_str();
            let suffix = caps.get(2).unwrap().as_str();
            let octave = caps.get(3).unwrap().as_str();
            pc = pitch_class(letter.chars().next().unwrap()) + parse_adjustment(suffix);
            o = octave.parse().unwrap();
        }
        Pitch {
            pitch_class: pc,
            octave: o,
        }
    }
    fn string(&self, bias: Bias) -> String {
        format!("{}{}", self.note(bias).string(), self.octave)
    }
    fn note(&self, bias: Bias) -> Note {
        Note {
            spelling: pitch_class_to_letter(self.pitch_class, bias),
            pitch_class: self.pitch_class,
        }
    }
    fn frequency(&self) -> f64 {
        return 0.0;
    }
}

fn circle_of_fifths() -> Vec<Note> {
    [
        "C", "F", "Bb", "Eb", "Ab", "Db", "Gb", "F#", "B", "E", "A", "D", "G",
    ]
    .iter()
    .map(|x| Note::parse(*x))
    .collect()
}
struct Note {
    spelling: char,
    pitch_class: i8,
}

impl Note {
    fn string(&self) -> String {
        match self.adjustment() {
            2 => format!("{}𝄪", self.spelling),
            1 => format!("{}♯", self.spelling),
            0 => String::from(self.spelling),
            -1 => format!("{}♭", self.spelling),
            -2 => format!("{}𝄫", self.spelling),
            _ => String::from("invalid"), // TODO
        }
    }

    fn add(&self, adjustment: i8, spelling: char) -> Note {
        Note {
            spelling,
            pitch_class: (self.pitch_class + adjustment) % 12,
        }
    }

    fn incr(&self, interval: Interval, bias: Bias) -> Note {
        let pitch_class = self.pitch_class + interval as i8;
        Note {
            pitch_class,
            spelling: pitch_class_to_letter(pitch_class, bias),
        }
    }

    fn adjustment(&self) -> i8 {
        if self.pitch_class == 11 && self.spelling == 'C' {
            -1
        } else {
            self.pitch_class - pitch_class(self.spelling)
        }
    }

    fn parse(s: &str) -> Note {
        let (first_char, suffix) = s.split_at(1);
        let spelling = first_char.chars().next().unwrap();
        let adjustment = parse_adjustment(suffix);
        Note {
            spelling,
            pitch_class: (pitch_class(spelling) + adjustment) % 12,
        }
    }

    fn new(spelling: char, pitch_class: i8) -> Note {
        Note {
            spelling,
            pitch_class: pitch_class % 12,
        }
    }
}

fn chromatic(p: Pitch, n: i8) -> Vec<Pitch> {
    let mut v = Vec::new();
    for i in 0..n {
        v.push(Pitch {
            pitch_class: (p.pitch_class + i) % 12,
            octave: (p.pitch_class + i) / 12,
        })
    }
    v
}

fn main_page_handler<'a>(key: console::Key) -> Action<'a> {
    match key {
        console::Key::Char('1') => Action::Render(Page {
            text: "1\n2\n3\n4\n5\n6\n7\n8\n9\n10",
            handler: main_page_handler,
        }),
        console::Key::Char('2') => Action::Render(Page {
            text: "A\nB\nC\nD\nE\nF\nG\nH\nI\nJ",
            handler: main_page_handler,
        }),
        console::Key::Char('q') | console::Key::Char('Q') => Action::Destroy,
        _ => Action::Noop,
    }
}

fn main_page<'a>() -> Page<'a> {
    Page {
        text: "=== Music Theory Quiz Game ===\n\
        \n\
        What would you like to do?\n\
        \n\
        [1] Intervals\n\
        [2] Scale Types\n\
        [3] Chords\n\
        \n\
        [q] Quit",
        handler: main_page_handler,
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let circle_of_fifths = circle_of_fifths();

    let mut term = Term::buffered_stdout();
    let mut screen = QuizScreen::fullscreen(&term);

    screen.init().unwrap();
    screen.run(&main_page()).unwrap();
    screen.destroy().unwrap();
}

struct QuizScreen<'a> {
    term: &'a console::Term,
    row: usize,
    col: usize,
    width: usize,
    height: usize,
}

enum Action<'a> {
    Noop,
    Render(Page<'a>),
    Destroy,
}

struct Page<'a> {
    text: &'a str,
    handler: fn(console::Key) -> Action<'a>,
}

impl QuizScreen<'_> {
    fn fullscreen(mut term: &console::Term) -> QuizScreen {
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

    fn run(&mut self, start: &Page) -> io::Result<()> {
        self.render(start)?;
        let mut handler = start.handler;
        loop {
            match (handler)(self.term.read_key()?) {
                Action::Render(page) => {
                    handler = page.handler;
                    self.render(&page)
                }
                Action::Destroy => break,
                Action::Noop => Ok(()),
            }?;
        }
        Ok(())
    }

    fn render(&mut self, page: &Page) -> io::Result<()> {
        self.term.clear_screen()?;
        self.border()?;
        self.write_page(page)?;
        self.term.flush()?;
        Ok(())
    }

    fn write_page(&mut self, page: &Page) -> io::Result<()> {
        for (i, line) in page.text.split("\n").enumerate() {
            self.term.move_cursor_to(4, i + 2)?;
            self.term.write(line.as_bytes())?;
        }
        Ok(())
    }

    fn border(&self) -> io::Result<()> {
        let mut term = self.term;

        let side = "┃".as_bytes();
        let top = "━".as_bytes();
        let tl = "┏".as_bytes();
        let bl = "┗".as_bytes();
        let tr = "┓".as_bytes();
        let br = "┛".as_bytes();

        for col in 0..(self.width - 1) {
            term.move_cursor_to(col, 0)?;
            term.write(top)?;
            term.move_cursor_to(col, self.height - 1)?;
            term.write(top)?;
        }

        for row in 0..(self.height - 1) {
            term.move_cursor_to(0, row)?;
            term.write(side)?;
            term.move_cursor_to(self.width - 1, row)?;
            term.write(side)?;
        }

        term.move_cursor_to(0, 0)?;
        term.write(tl)?;
        term.move_cursor_to(0, self.height)?;
        term.write(bl)?;
        term.move_cursor_to(self.width - 1, 0)?;
        term.write(tr)?;
        term.move_cursor_to(self.width - 1, self.height)?;
        term.write(br)?;
        Ok(())
    }
}
