use crate::fraction::Fraction;

/// Elements of a phrase, contains either a note or a chord.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhraseElement {
    Note(Note),
    Chord(Vec<Note>),
}

impl PhraseElement {
    /// Check if an element contains a note of the same pitch as the given one.
    pub fn contains_note(&self, note: Note) -> bool {
        match self {
            PhraseElement::Chord(c) => c.iter().any(|n| n.pitch_equals(&note)),
            PhraseElement::Note(n) => n.pitch_equals(&note),
        }
    }

    /// Checks if a note in the element of a given pitch has a start tie, and return the note if it does.
    pub fn has_start_tie(&mut self, note: Note) -> Option<&mut Note> {
        match self {
            PhraseElement::Chord(c) => {
                if let Some(n) = c.iter_mut().find(|n| n.pitch_equals(&note)) {
                    return if n.tie.is_start() { Some(n) } else { None };
                }
                None
            }
            PhraseElement::Note(n) => {
                if n.pitch_equals(&note) && n.tie.is_start() {
                    Some(n)
                } else {
                    None
                }
            }
        }
    }

    /// Checks if a note in the element of a given pitch has a stop tie, and return the note if it does.
    pub fn has_stop_tie(&mut self, note: Note) -> Option<&mut Note> {
        match self {
            PhraseElement::Chord(c) => {
                if let Some(n) = c.iter_mut().find(|n| n.pitch_equals(&note)) {
                    return if n.tie.is_stop() { Some(n) } else { None };
                }
                None
            }
            PhraseElement::Note(n) => {
                if n.pitch_equals(&note) && n.tie.is_stop() {
                    Some(n)
                } else {
                    None
                }
            }
        }
    }

    /// Merge a note into the element.
    pub fn merge_note(&mut self, note: Note) {
        match self {
            PhraseElement::Note(n) => {
                if note.pitch_equals(n) {
                    if note.tie.is_start() {
                        n.tie.start();
                    }
                    if note.tie.is_stop() {
                        n.tie.stop();
                    }
                } else {
                    let new_chord = vec![*n, note];
                    *self = PhraseElement::Chord(new_chord);
                }
            }
            PhraseElement::Chord(c) => match c.iter_mut().find(|n| n.pitch_equals(&note)) {
                Some(n) => {
                    if note.pitch_equals(n) {
                        if note.tie.is_start() {
                            n.tie.start();
                        }
                        if note.tie.is_stop() {
                            n.tie.stop();
                        }
                    }
                }
                None => c.push(note),
            },
        }
    }

    /// Add a start tie to every note in the element.
    pub fn start_tie(&mut self) {
        match self {
            PhraseElement::Chord(notes) => {
                for note in notes {
                    note.tie.start();
                }
            }
            PhraseElement::Note(note) => note.tie.start(),
        }
    }

    /// Add a stop tie to every note in the element.
    pub fn stop_tie(&mut self) {
        match self {
            PhraseElement::Chord(notes) => {
                for note in notes {
                    note.tie.stop();
                }
            }
            PhraseElement::Note(note) => note.tie.stop(),
        }
    }

    /// Get the total value and number of notes in an element which can be used to calculate the mean.
    pub fn mean(&self) -> (u8, u8) {
        match self {
            PhraseElement::Note(n) => (n.value(), 1),
            PhraseElement::Chord(c) => (
                c.iter().fold(0, |sum, note| sum + note.value()),
                c.len() as u8,
            ),
        }
    }

    /// Get the value of the minimum pitch in an element.
    pub fn min(&self) -> u8 {
        match self {
            PhraseElement::Note(n) => n.value(),
            PhraseElement::Chord(c) => c.iter().map(|n| n.value()).min().unwrap(),
        }
    }

    /// Get the value of the maximum pitch in an element.
    pub fn max(&self) -> u8 {
        match self {
            PhraseElement::Note(n) => n.value(),
            PhraseElement::Chord(c) => c.iter().map(|n| n.value()).max().unwrap(),
        }
    }

    /// Transpose each note in an element by a given number of octaves.
    pub fn transpose_octaves(&mut self, octaves: i8) {
        match self {
            PhraseElement::Note(n) => n.octave = (n.octave as i8 + octaves) as u8,
            PhraseElement::Chord(c) => {
                for n in c {
                    n.octave = (n.octave as i8 + octaves) as u8
                }
            }
        }
    }
}

/// Defines a note.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Note {
    pub step: NoteName,
    pub octave: u8,
    pub alter: i8,
    pub tie: Tie,
}

impl Note {
    /// Create a new note object.
    pub fn new(step: NoteName, octave: u8, alter: i8, tie: Tie) -> Self {
        Note {
            step,
            octave,
            alter,
            tie,
        }
    }

    /// Get the value of a note.
    pub fn value(&self) -> u8 {
        let alter = self.octave as i8 * 12 + self.alter;
        (self.step.value() as i8 + alter) as u8
    }

    /// Check if the other note has the same pitch as this note.
    pub fn pitch_equals(&self, other: &Note) -> bool {
        self.alter == other.alter && self.octave == other.octave && self.step == other.step
    }

    /// Remove the start tie from the note.
    pub fn remove_start_tie(&mut self) {
        self.tie = match self.tie {
            Tie::Start | Tie::None => Tie::None,
            Tie::StartStop | Tie::Stop => Tie::Stop,
        };
    }

    /// Remove the stop tie from the note.
    pub fn remove_stop_tie(&mut self) {
        self.tie = match self.tie {
            Tie::Stop | Tie::None => Tie::None,
            Tie::StartStop | Tie::Start => Tie::Start,
        };
    }
}

/// Defines a tie.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Tie {
    None,
    Start,
    Stop,
    StartStop,
}

impl Tie {
    /// Ensure the tie contains a start tie.
    pub fn start(&mut self) {
        *self = match self {
            Tie::None | Tie::Start => Tie::Start,
            Tie::StartStop | Tie::Stop => Tie::StartStop,
        }
    }

    /// Ensure the tie contains a stop tie.
    pub fn stop(&mut self) {
        *self = match self {
            Tie::None | Tie::Stop => Tie::Stop,
            Tie::Start | Tie::StartStop => Tie::StartStop,
        }
    }

    /// Check if the tie contains a start tie.
    pub fn is_start(&self) -> bool {
        match self {
            Tie::Start | Tie::StartStop => true,
            _ => false,
        }
    }

    /// Check if the tie contains a stop tie.
    pub fn is_stop(&self) -> bool {
        match self {
            Tie::Stop | Tie::StartStop => true,
            _ => false,
        }
    }
}

/// Defines the note names.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum NoteName {
    A = 5,
    B,
    C = 0,
    D,
    E,
    F,
    G,
}

impl NoteName {
    /// Get a note name from a string.
    pub fn parse(name: &str) -> Option<NoteName> {
        Some(match name {
            "A" => NoteName::A,
            "B" => NoteName::B,
            "C" => NoteName::C,
            "D" => NoteName::D,
            "E" => NoteName::E,
            "F" => NoteName::F,
            "G" => NoteName::G,
            _ => return None,
        })
    }

    /// Get the base value of a note.
    fn value(&self) -> u8 {
        match self {
            NoteName::A => 9,
            NoteName::B => 11,
            NoteName::C => 0,
            NoteName::D => 2,
            NoteName::E => 4,
            NoteName::F => 5,
            NoteName::G => 7,
        }
    }

    /// Get the note name from its index.
    pub fn from_index(index: u8) -> Option<Self> {
        Some(match index {
            0 => NoteName::C,
            1 => NoteName::D,
            2 => NoteName::E,
            3 => NoteName::F,
            4 => NoteName::G,
            5 => NoteName::A,
            6 => NoteName::B,
            _ => return None,
        })
    }

    /// Get the name of a note.
    pub fn name(&self) -> &str {
        match self {
            NoteName::A => "A",
            NoteName::B => "B",
            NoteName::C => "C",
            NoteName::D => "D",
            NoteName::E => "E",
            NoteName::F => "F",
            NoteName::G => "G",
        }
    }
}

/// Defines a note type.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum NoteType {
    N1024th,
    N512th,
    N256th,
    N128th,
    N64th,
    N32nd,
    N16th,
    Eighth,
    Quarter,
    Half,
    Whole,
    Breve,
    Long,
    Maxima,
}

impl NoteType {
    /// Get the bar divisions given the smallest note.
    pub fn to_divisions(&self) -> u32 {
        match self {
            NoteType::N1024th => 256,
            NoteType::N512th => 128,
            NoteType::N256th => 64,
            NoteType::N128th => 32,
            NoteType::N64th => 16,
            NoteType::N32nd => 8,
            NoteType::N16th => 4,
            NoteType::Eighth => 2,
            NoteType::Quarter => 1,
            NoteType::Half => 1,
            NoteType::Whole => 1,
            NoteType::Breve => 1,
            NoteType::Long => 1,
            NoteType::Maxima => 1,
        }
    }

    /// Get the length of a note in divisions given the bar divisions.
    pub fn divisions(&self, current: u32) -> u32 {
        let division = match self {
            NoteType::N1024th => 8192,
            NoteType::N512th => 4096,
            NoteType::N256th => 2048,
            NoteType::N128th => 1024,
            NoteType::N64th => 512,
            NoteType::N32nd => 256,
            NoteType::N16th => 128,
            NoteType::Eighth => 64,
            NoteType::Quarter => 32,
            NoteType::Half => 16,
            NoteType::Whole => 8,
            NoteType::Breve => 4,
            NoteType::Long => 2,
            NoteType::Maxima => 1,
        };
        (current * 32) / division
    }

    /// Get the musicXML name of the note.
    pub fn name(&self) -> &str {
        match self {
            NoteType::N1024th => "1024th",
            NoteType::N512th => "512th",
            NoteType::N256th => "256th",
            NoteType::N128th => "128th",
            NoteType::N64th => "64th",
            NoteType::N32nd => "32nd",
            NoteType::N16th => "16th",
            NoteType::Eighth => "eighth",
            NoteType::Quarter => "quarter",
            NoteType::Half => "half",
            NoteType::Whole => "whole",
            NoteType::Breve => "breve",
            NoteType::Long => "long",
            NoteType::Maxima => "maxima",
        }
    }

    /// Parse a MusicXML note type.
    pub fn parse(name: &str) -> Option<NoteType> {
        Some(match name {
            "1024th" => NoteType::N1024th,
            "512th" => NoteType::N512th,
            "256th" => NoteType::N256th,
            "128th" => NoteType::N128th,
            "64th" => NoteType::N64th,
            "32nd" => NoteType::N32nd,
            "16th" => NoteType::N16th,
            "eighth" => NoteType::Eighth,
            "quarter" => NoteType::Quarter,
            "half" => NoteType::Half,
            "whole" => NoteType::Whole,
            "breve" => NoteType::Breve,
            "long" => NoteType::Long,
            "maxima" => NoteType::Maxima,
            _ => return None,
        })
    }

    /// Get the value of the note type as a fraction.
    pub fn get_value(&self) -> Fraction {
        match self {
            NoteType::N1024th => Fraction::new(1, 256),
            NoteType::N512th => Fraction::new(1, 128),
            NoteType::N256th => Fraction::new(1, 64),
            NoteType::N128th => Fraction::new(1, 32),
            NoteType::N64th => Fraction::new(1, 16),
            NoteType::N32nd => Fraction::new(1, 8),
            NoteType::N16th => Fraction::new(1, 4),
            NoteType::Eighth => Fraction::new(1, 2),
            NoteType::Quarter => Fraction::new(1, 1),
            NoteType::Half => Fraction::new(2, 1),
            NoteType::Whole => Fraction::new(4, 1),
            NoteType::Breve => Fraction::new(8, 1),
            NoteType::Long => Fraction::new(16, 1),
            NoteType::Maxima => Fraction::new(32, 1),
        }
    }

    /// Get a note type from its length in bar divisions.
    fn from_duration(duration: u32, divisions: u32) -> Option<NoteType> {
        let balanced = Fraction::balance(duration as i32, divisions as i32);
        Some(match balanced {
            (1, 256) => NoteType::N1024th,
            (1, 128) => NoteType::N512th,
            (1, 64) => NoteType::N256th,
            (1, 32) => NoteType::N128th,
            (1, 16) => NoteType::N64th,
            (1, 8) => NoteType::N32nd,
            (1, 4) => NoteType::N16th,
            (1, 2) => NoteType::Eighth,
            (1, 1) => NoteType::Quarter,
            (2, 1) => NoteType::Half,
            (4, 1) => NoteType::Whole,
            (8, 1) => NoteType::Breve,
            (16, 1) => NoteType::Long,
            (32, 1) => NoteType::Maxima,
            _ => return None,
        })
    }

    /// Get a list of note types from a fraction
    pub fn from_fraction(mut fraction: Fraction) -> Vec<NoteType> {
        debug_assert!(fraction >= Fraction::zero());
        let mut notes = Vec::new();
        let mut chunk = Fraction::new(32, 1);
        while !fraction.is_zero() {
            if chunk <= fraction {
                fraction -= chunk;
                notes.push(
                    Self::from_duration(chunk.numerator() as u32, chunk.denominator() as u32)
                        .unwrap(),
                );
            }
            chunk /= Fraction::new(2, 1);
        }
        notes
    }
}

/// Defines a Clef.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Clef {
    Treble,
    Bass,
}

impl Clef {
    /// Get the MusicXML sign for a clef.
    pub fn sign(&self) -> &str {
        match self {
            Clef::Treble => "G",
            Clef::Bass => "F",
        }
    }

    /// Get the MusicXML line for a clef.
    pub fn line(&self) -> &str {
        match self {
            Clef::Treble => "2",
            Clef::Bass => "4",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fraction::Fraction;
    use crate::phrase_element::{Note, NoteName, NoteType, Tie};

    #[test]
    fn note_values() {
        let note = Note::new(NoteName::C, 4, 0, Tie::None);
        assert_eq!(note.value(), 48);

        let note = Note::new(NoteName::B, 3, 0, Tie::None);
        assert_eq!(note.value(), 47);

        let note = Note::new(NoteName::C, 4, 1, Tie::None);
        assert_eq!(note.value(), 49);
    }

    #[test]
    fn note_types_from_fraction() {
        let duration = Fraction::new(1, 1);
        assert_eq!(NoteType::from_fraction(duration), vec![NoteType::Quarter]);

        let duration = Fraction::new(3, 2);
        assert_eq!(
            NoteType::from_fraction(duration),
            vec![NoteType::Quarter, NoteType::Eighth]
        );

        let duration = Fraction::zero();
        assert_eq!(NoteType::from_fraction(duration), vec![]);
    }
}
