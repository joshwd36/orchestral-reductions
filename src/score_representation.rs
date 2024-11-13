use crate::fraction::Fraction;
use crate::phrase::Phrase;
use crate::phrase_element::*;
use itertools::Itertools;
use roxmltree::{Document, Node};
use std::collections::BTreeMap;

/// Parses a MusicXML document to a PhraseList.
pub struct ScoreParser<'a> {
    doc: Document<'a>,
}

impl<'a> ScoreParser<'a> {
    /// Construct a musicXML parser with a roxmltree document.
    pub fn new(doc: Document) -> ScoreParser {
        ScoreParser { doc }
    }

    /// Parse the score.
    pub fn parse_score(&mut self, phrase_limit: u32) -> PhraseList {
        let mut score = PhraseList::new();
        let score_element = self.doc.root_element();
        let children = score_element.children();
        let parts = children.filter(|n| n.has_tag_name("part"));
        for part in parts {
            score.parse_part(part, phrase_limit);
        }

        score
    }
}

/// Defines a list of phrases.
#[derive(Debug)]
pub struct PhraseList {
    phrases: Vec<Phrase>,
    keys: BTreeMap<Fraction, i8>,
    times: BTreeMap<Fraction, (u8, u8)>,
}


impl PhraseList {
    fn new() -> PhraseList {
        PhraseList {
            phrases: Vec::new(),
            keys: BTreeMap::new(),
            times: BTreeMap::new(),
        }
    }

    /// Parse a MusicXML part into a list of phrases.
    fn parse_part(&mut self, part: Node, phrase_limit: u32) {
        let measures = part.children().filter(|n| n.has_tag_name("measure"));
        let mut divisions: u32 = 0;
        let mut current_pos = Fraction::zero();
        let mut note_list: BTreeMap<Fraction, (PhraseElement, Fraction)> = BTreeMap::new();
        let mut current_transpose = Transpose::new();
        let mut last_bar_num = 0;
        for measure in measures {
            // Parse attribute elements.
            if let Some(attributes) = measure.children().find(|n| n.has_tag_name("attributes")) {
                self.parse_attributes(
                    &attributes,
                    current_pos,
                    &mut divisions,
                    &mut current_transpose,
                );
            }
            // Parse the bar number.
            let bar_num = measure
                .attribute("number")
                .map(|n| n.parse::<u32>().ok())
                .flatten()
                .unwrap_or_default();
            // End phrase if longer than phrase limit.
            if phrase_limit > 0 {
                if bar_num >= last_bar_num + phrase_limit && !note_list.is_empty() {
                    self.phrases.push(Phrase::new(note_list));
                    note_list = BTreeMap::new();
                    last_bar_num = bar_num;
                }
            }
            // Parse the notes.
            let notes = measure.children().filter(|n| n.has_tag_name("note"));
            for note in notes {
                // PArse the duration.
                let duration = match note
                    .children()
                    .find(|n| n.has_tag_name("duration"))
                    .map(|n| n.text().unwrap().parse().ok())
                    .flatten()
                {
                    Some(duration) => Fraction::new(duration, divisions as i32),
                    None => {
                        let note_type = note
                            .children()
                            .find(|n| n.has_tag_name("type"))
                            .map(|n| NoteType::parse(n.text().unwrap()))
                            .flatten()
                            .unwrap();
                        let dots = note.children().filter(|n| n.has_tag_name("dot")).count() as u32;
                        let base_value = note_type.get_value();
                        base_value * Fraction::new(3i32.pow(dots), 2i32.pow(dots))
                    }
                };

                // Parse the ties.
                let mut tie = Tie::None;
                if let Some(notations) = note.children().find(|n| n.has_tag_name("notations")) {
                    let ties = notations.children().filter(|n| n.has_tag_name("tied"));
                    for t in ties {
                        if let Some(t) = t.attribute("type") {
                            match t {
                                "start" => {
                                    tie.start();
                                }
                                "stop" => {
                                    tie.stop();
                                }
                                "continue" => {
                                    tie.stop();
                                    tie.start();
                                }
                                _ => (),
                            }
                        }
                    }
                }

                let chord = note.children().find(|n| n.has_tag_name("chord")).is_some();

                // Parse the pitch.
                let pitch = note.children().find(|n| n.has_tag_name("pitch"));
                match pitch {
                    Some(pitch) => {
                        let step = pitch
                            .children()
                            .find(|n| n.has_tag_name("step"))
                            .map(|n| NoteName::parse(n.text().unwrap()))
                            .unwrap()
                            .unwrap();
                        let octave = pitch
                            .children()
                            .find(|n| n.has_tag_name("octave"))
                            .unwrap()
                            .text()
                            .and_then(|n| n.parse().ok())
                            .unwrap();
                        let alter = pitch
                            .children()
                            .find(|n| n.has_tag_name("alter"))
                            .and_then(|n| n.text())
                            .and_then(|n| n.parse().ok())
                            .or(Some(0))
                            .unwrap();

                        let mut note = Note::new(step, octave, alter, tie);
                        current_transpose.apply(&mut note);

                        // If its a chord, add it to a new phrase.
                        if chord {
                            current_pos -= duration;
                            let mut note_list = BTreeMap::new();
                            note_list.insert(current_pos, (PhraseElement::Note(note), duration));
                            self.phrases.push(Phrase::new(note_list));
                        } else {
                            note_list.insert(current_pos, (PhraseElement::Note(note), duration));
                        }
                    }
                    // IF its a rest, end the phrase and start a new one.
                    None => {
                        if let Some(_) = note.children().find(|n| n.has_tag_name("rest")) {
                            if !note_list.is_empty() {
                                self.phrases.push(Phrase::new(note_list));
                                note_list = BTreeMap::new();
                                last_bar_num = bar_num;
                            }
                        }
                    }
                }

                current_pos += duration;
            }
        }
        if !note_list.is_empty() {
            self.phrases.push(Phrase::new(note_list));
        }
    }

    /// Parse the measure attributes.
    fn parse_attributes(
        &mut self,
        attributes: &Node,
        current_pos: Fraction,
        divisions: &mut u32,
        current_transpose: &mut Transpose,
    ) {
        // Get the divisions.
        let attributes_divisions = attributes.children().find(|n| n.has_tag_name("divisions"));
        if let Some(attributes_divisions) = attributes_divisions {
            *divisions = attributes_divisions.text().unwrap().parse().unwrap()
        }

        // Parse the transpostion.
        if let Some(transpose) = attributes.children().find(|n| n.has_tag_name("transpose")) {
            current_transpose.chromatic = transpose
                .children()
                .find(|n| n.has_tag_name("chromatic"))
                .map(|x| x.text().unwrap().parse().unwrap())
                .unwrap_or_default();
            current_transpose.diatonic = transpose
                .children()
                .find(|n| n.has_tag_name("diatonic"))
                .map(|x| x.text().unwrap().parse().unwrap())
                .unwrap_or_default();
            current_transpose.octave = transpose
                .children()
                .find(|n| n.has_tag_name("octave-change"))
                .map(|x| x.text().unwrap().parse().unwrap())
                .unwrap_or_default();
        }

        // Parse the key signature.
        let key = attributes.children().find(|n| n.has_tag_name("key"));
        if let Some(key) = key {
            let fifths: i32 = key
                .children()
                .find(|n| n.has_tag_name("fifths"))
                .unwrap()
                .text()
                .unwrap()
                .parse()
                .unwrap();

            let transposed_key = ((fifths + current_transpose.chromatic * 7) % 12) as i8;
            let existing = self.keys.insert(current_pos, transposed_key);
            match existing {
                Some(n) => {
                    if transposed_key != n {
                        panic!("Conflicting key signatures at {:?}", current_pos);
                    }
                }
                None => (),
            }
        }

        // Parse the time signature.
        let time = attributes.children().find(|n| n.has_tag_name("time"));
        if let Some(time) = time {
            let beats = time
                .children()
                .find(|n| n.has_tag_name("beats"))
                .unwrap()
                .text()
                .unwrap()
                .parse()
                .unwrap();
            let beat_type = time
                .children()
                .find(|n| n.has_tag_name("beat-type"))
                .unwrap()
                .text()
                .unwrap()
                .parse()
                .unwrap();
            let existing = self.times.insert(current_pos, (beats, beat_type));
            match existing {
                Some(n) => {
                    if (beats, beat_type) != n {
                        panic!("Conflicting time signatures at {:?}", current_pos);
                    }
                }
                None => (),
            }
        }
    }

    /// Distribute the phrases onto staves.
    pub fn distribute_staves(mut self, staves: u8) -> StaveList {
        self.phrases.sort_unstable_by_key(|a| a.start());
        let mut new_staves = vec![Vec::new(); staves as usize];

        // For every phrase, get the maximum and minimum pitches at the start position, and split evenly for each phrase. Then allocate phrase based on which stave it's closest to.
        for phrase in &self.phrases {
            if phrase.num_elements() > 0 {
                let start = phrase.start();
                let (first_element, _) = phrase.first();
                let (sum, total) = first_element.mean();
                let start_mean = sum / total;
                let max = self
                    .phrases
                    .iter()
                    .filter_map(|p| p.max_at(start))
                    .max()
                    .unwrap();
                let min = self
                    .phrases
                    .iter()
                    .filter_map(|p| p.min_at(start))
                    .min()
                    .unwrap();

                let midpoints = (0..staves).rev().map(|i| {
                    let split_size = (max - min) / (staves + 1);
                    (i + 1) * split_size + min
                });
                let (index, _) = midpoints
                    .enumerate()
                    .min_by_key(|(_, i)| (start_mean as i32 - *i as i32).abs())
                    .unwrap();
                new_staves[index].push(phrase.clone());
            }
        }

        StaveList {
            staves: new_staves,
            keys: self.keys,
            times: self.times,
        }
    }

    /// Merge phrases into staves by keeping an average pitch of each stave and allocating based on which stave its closest to.
    pub fn merge_by_average(mut self, staves: u8) -> StaveList {
        self.phrases.sort_unstable_by_key(|a| a.start());
        let mut averages = Vec::with_capacity(staves as usize);
        let max = self
            .phrases
            .iter()
            .filter_map(|p| p.max_at(Fraction::zero()))
            .max()
            .unwrap_or(96);
        let min = self
            .phrases
            .iter()
            .filter_map(|p| p.min_at(Fraction::zero()))
            .min()
            .unwrap_or(9);
        for i in (0..staves).rev() {
            let split_size = (max - min) / (staves + 1);
            let value = (i + 1) * split_size + min;
            averages.push(value);
        }
        let mut new_phrases = vec![vec![Phrase::default()]; staves as usize];
        for phrase in self.phrases {
            let (first_element, _) = phrase.first();
            let (sum, total) = first_element.mean();
            let start_mean = sum / total;
            let (index, _) = averages
                .iter()
                .enumerate()
                .min_by_key(|(_, i)| (start_mean as i32 - **i as i32).abs())
                .unwrap();
            averages[index] = phrase.mean();
            new_phrases[index][0].merge(phrase);
        }
        StaveList {
            staves: new_phrases,
            keys: self.keys,
            times: self.times,
        }
    }
}

/// Represents a list of staves, with each stave containing a list of phrases.
#[derive(Debug)]
pub struct StaveList {
    pub staves: Vec<Vec<Phrase>>,
    pub keys: BTreeMap<Fraction, i8>,
    pub times: BTreeMap<Fraction, (u8, u8)>,
}

impl StaveList {
    /// Check if the phrase can have the phrases based on the largest interval.
    fn can_have_phrase(stave: &Vec<Phrase>, phrase: &Phrase, largest_stretch: u32) -> bool {
        for &position in phrase.elements_ref().keys() {
            if let Some(stave_max) = stave
                .iter()
                .filter_map(|phrase| phrase.max_at(position))
                .max()
            {
                let stave_min = stave
                    .iter()
                    .filter_map(|phrase| phrase.min_at(position))
                    .min()
                    .unwrap();
                let phrase_max = phrase.max_at(position).unwrap();
                let phrase_min = phrase.min_at(position).unwrap();
                if stave_max.max(phrase_max) - stave_min.min(phrase_min) > largest_stretch as u8 {
                    return false;
                }
            }
        }
        true
    }

    /// Transpose the octaves of phrases within each stave to try and be within the given maximum interval.
    pub fn adjust_octaves(&mut self, largest_stretch: u32) {
        let num_staves = self.staves.len();
        for i in 0..num_staves {
            let (mut previous, stave, mut next) = get_surrounding_mut(&mut self.staves, i);
            stave.sort_unstable_by_key(|a| a.start());
            let positions: Vec<Fraction> = stave
                .iter()
                .map(|p| p.elements_ref().iter().map(|(start, _)| *start))
                .kmerge()
                .unique()
                .collect();

            for position in positions {
                loop {
                    //println!("position: {} stave: {}", position, i);
                    if let Some((max_phrase, max_val)) = stave
                        .iter()
                        .enumerate()
                        .filter_map(|(i, p)| p.max_at(position).map(|m| (i, m as u32)))
                        .max_by_key(|(_, m)| *m)
                    {
                        let (min_phrase, min_val) = stave
                            .iter()
                            .enumerate()
                            .filter_map(|(i, p)| p.min_at(position).map(|m| (i, m as u32)))
                            .min_by_key(|(_, m)| *m)
                            .unwrap();
                        let (total, count) = stave.iter().filter_map(|p| p.mean_at(position)).fold(
                            (0, 0),
                            |(total, count), (el_total, el_count)| {
                                (total + el_total as u32, count + el_count as u32)
                            },
                        );

                        let mean = total / count;
                        let midpoint = (min_val + max_val) / 2;
                        if max_val - min_val > largest_stretch {
                            let mut moved = false;
                            if let Some(previous) = &mut previous {
                                if Self::can_have_phrase(
                                    previous,
                                    &stave[max_phrase],
                                    largest_stretch,
                                ) {
                                    previous.push(stave.remove(max_phrase));
                                    moved = true;
                                }
                            }
                            if !moved {
                                let other_max = stave
                                    .iter()
                                    .enumerate()
                                    .filter_map(
                                        |(_num, p)| {
                                            p.max_at(position)
                                                .filter(|pitch| *pitch as u32 != max_val)
                                        }, //p.max_at(position).filter(|_| num != max_phrase)
                                    )
                                    .max()
                                    .unwrap()
                                    as u32;
                                let other_min = stave
                                    .iter()
                                    .enumerate()
                                    .filter_map(
                                        |(_num, p)| {
                                            p.min_at(position)
                                                .filter(|pitch| *pitch as u32 != min_val)
                                        }, //p.min_at(position).filter(|_| num != min_phrase)
                                    )
                                    .min()
                                    .unwrap()
                                    as u32;

                                if mean < midpoint
                                    && (i != 0 || (max_val - 12 >= other_max))
                                    && stave[max_phrase].min_val() >= 21
                                    && (i != num_staves - 1 || max_val - 12 >= min_val)
                                {
                                    stave[max_phrase].transpose_octaves(-1);
                                } else {
                                    if let Some(next) = &mut next {
                                        next.push(stave.remove(min_phrase));
                                    } else if (i != num_staves - 1 || min_val + 12 <= other_min)
                                        && stave[min_phrase].max_val() <= 84
                                        && (i != 0 || min_val + 12 <= max_val)
                                    {
                                        stave[min_phrase].transpose_octaves(1);
                                    } else if i == 0 {
                                        stave.remove(min_phrase);
                                    } else {
                                        stave.remove(max_phrase);
                                    }
                                }
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }

    /// Merge all phrases on a stave together.
    pub fn merge(self) -> Self {
        let mut new_staves = Vec::new();
        for stave in self.staves {
            let mut new_phrase = Phrase::default();
            for phrase in stave {
                new_phrase.merge(phrase);
            }
            new_staves.push(vec![new_phrase]);
        }
        Self {
            times: self.times,
            keys: self.keys,
            staves: new_staves,
        }
    }
}

/// Contains information about instrument transposition.
struct Transpose {
    chromatic: i32,
    diatonic: i32,
    octave: i8,
}

impl Transpose {
    /// Create a new transformation object.
    fn new() -> Self {
        Self {
            chromatic: 0,
            diatonic: 0,
            octave: 0,
        }
    }

    /// Apply a transformation.
    fn apply(&self, note: &mut Note) {
        let original_value = note.value() as i32;
        let mut step_change = note.step as i32 + self.diatonic;
        while step_change < 0 {
            step_change += 7;
            note.octave -= 1;
        }
        while step_change >= 7 {
            step_change -= 7;
            note.octave += 1;
        }
        note.step = NoteName::from_index((step_change % 7) as u8).unwrap();
        note.octave += (step_change / 7) as u8;
        let difference = note.value() as i32 - original_value;
        note.alter -= (difference - self.chromatic) as i8;
        note.octave = (note.octave as i8 + self.octave) as u8;
    }
}

/// Get the surrounding elements in a slice, if they exist.
fn get_surrounding_mut<T>(slice: &mut [T], i: usize) -> (Option<&mut T>, &mut T, Option<&mut T>) {
    let (first, others) = slice.split_at_mut(i);
    let (second, third) = others.split_at_mut(1);
    (
        first.last_mut(),
        second.first_mut().unwrap(),
        third.first_mut(),
    )
}

#[cfg(test)]
mod tests {
    use crate::phrase_element::{Note, NoteName, Tie};
    use crate::score_representation::Transpose;

    #[test]
    fn transpose_ordinary() {
        let mut transpose = Transpose::new();
        transpose.chromatic = -2;
        transpose.diatonic = -1;

        // test ordinary transposition
        let mut note = Note::new(NoteName::A, 4, 0, Tie::None);
        transpose.apply(&mut note);
        let expected = Note::new(NoteName::G, 4, 0, Tie::None);
        assert_eq!(note, expected);
    }

    #[test]
    fn transpose_alteration() {
        let mut transpose = Transpose::new();
        transpose.chromatic = -2;
        transpose.diatonic = -1;

        // test transposition to a different alteration
        let mut note = Note::new(NoteName::F, 4, 0, Tie::None);
        transpose.apply(&mut note);
        let expected = Note::new(NoteName::E, 4, -1, Tie::None);
        assert_eq!(note, expected);
    }

    #[test]
    fn transpose_octave() {
        let mut transpose = Transpose::new();
        transpose.chromatic = -2;
        transpose.diatonic = -1;

        // test transposition over an octave
        let mut note = Note::new(NoteName::C, 4, 1, Tie::None);
        transpose.apply(&mut note);
        let expected = Note::new(NoteName::B, 3, 0, Tie::None);
        assert_eq!(note, expected);
    }
}
