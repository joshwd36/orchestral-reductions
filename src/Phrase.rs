use crate::fraction::Fraction;
use crate::phrase_element::{Note, PhraseElement};
use std::collections::BTreeMap;
use std::ops::Bound;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Phrase {
    elements: BTreeMap<Fraction, (PhraseElement, Fraction)>,
}

impl PartialOrd for Phrase {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Phrase {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let pitch1 = self.first().0.max();
        let pitch2 = other.first().0.max();
        pitch2.cmp(&pitch1)
    }
}

impl Phrase {
    pub fn min_duration(&self) -> Fraction {
        *(self
            .elements
            .iter()
            .map(|(_, (_, duration))| duration)
            .min()
            .unwrap())
    }

    pub(crate) fn new(elements: BTreeMap<Fraction, (PhraseElement, Fraction)>) -> Self {
        Phrase { elements }
    }

    pub fn elements(self) -> BTreeMap<Fraction, (PhraseElement, Fraction)> {
        self.elements
    }

    pub fn elements_ref(&self) -> &BTreeMap<Fraction, (PhraseElement, Fraction)> {
        &self.elements
    }

    pub fn start(&self) -> Fraction {
        *self.elements.keys().next().unwrap()
    }

    pub fn end(&self) -> Fraction {
        let (position, (_, length)) = self.elements.iter().rev().next().unwrap();
        *position + *length
    }

    pub fn first(&self) -> &(PhraseElement, Fraction) {
        self.elements.values().next().unwrap()
    }

    fn add_element(&mut self, element: PhraseElement, position: Fraction, duration: Fraction) {
        match element {
            PhraseElement::Chord(notes) => {
                for note in notes {
                    self.add_note(note, position, duration);
                }
            }
            PhraseElement::Note(note) => self.add_note(note, position, duration),
        }
    }

    fn add_note(&mut self, mut note: Note, position: Fraction, mut duration: Fraction) {
        if let Some(overlap_position) = self.next_overlap(position, duration) {
            let (overlap, _) = self.elements.get(&overlap_position).unwrap();
            if !(overlap.contains_note(note)) {
                let mut new_note = note;
                let new_duration = duration - (overlap_position - position);
                note.tie.start();
                new_note.tie.stop();
                self.add_note(new_note, overlap_position, new_duration);
            }
            duration = overlap_position - position;
        }
        match self.previous_overlap(position) {
            Some(overlap_position) => {
                let (overlap, len) = self.elements.remove(&overlap_position).unwrap();
                self.elements
                    .insert(position, (PhraseElement::Note(note), duration));
                self.add_element(overlap, overlap_position, len);
            }
            None => {
                let mut change_prev_tie = false;
                match self.elements.get_mut(&position) {
                    Some((element, len)) => {
                        let len = *len;
                        if len < duration {
                            let mut new_note = note;
                            new_note.tie.stop();
                            note.tie.start();
                            if let Some(tied) = element.has_stop_tie(note) {
                                tied.remove_stop_tie();
                                change_prev_tie = true;
                            }
                            element.merge_note(note);
                            let new_position = position + len;
                            let new_duration = duration - len;
                            self.elements.insert(
                                new_position,
                                (PhraseElement::Note(new_note), new_duration),
                            );
                        } else if len == duration {
                            if let Some(tied) = element.has_stop_tie(note) {
                                tied.remove_stop_tie();
                                change_prev_tie = true;
                            }
                            element.merge_note(note);
                        }
                        // len > duration
                        else {
                            let (old, _) = self
                                .elements
                                .insert(position, (PhraseElement::Note(note), duration))
                                .unwrap();
                            self.add_element(old, position, len);
                        }
                        if change_prev_tie {
                            if let Some((_, (previous, _))) =
                                self.elements.range_mut(..position).next()
                            {
                                if let Some(tied) = previous.has_start_tie(note) {
                                    tied.remove_start_tie();
                                }
                            }
                        }
                    }
                    None => {
                        self.elements
                            .insert(position, (PhraseElement::Note(note), duration));
                    }
                }
            }
        }
    }

    fn previous_overlap(&self, position: Fraction) -> Option<Fraction> {
        let (prev_position, (_, prev_duration)) = self.elements.range(..position).rev().next()?;
        if *prev_position + *prev_duration > position {
            Some(*prev_position)
        } else {
            None
        }
    }

    fn next_overlap(&self, position: Fraction, duration: Fraction) -> Option<Fraction> {
        let (&next_position, _) = self
            .elements
            .range((Bound::Excluded(position), Bound::Unbounded))
            .next()?;
        if position + duration > next_position {
            Some(next_position)
        } else {
            None
        }
    }

    pub fn merge(&mut self, other: Phrase) {
        for (position, (element, duration)) in other.elements {
            self.add_element(element, position, duration);
        }
    }

    pub fn split(self, split_point: Fraction) -> (Phrase, Phrase) {
        let mut phrase_1 = Phrase::default();
        let mut phrase_2 = Phrase::default();
        for (position, (mut element, length)) in self.elements() {
            if position + length <= split_point {
                phrase_1.add_element(element, position, length);
            } else if position < split_point && position + length > split_point {
                let mut element1 = element.clone();
                element1.start_tie();
                element.stop_tie();
                phrase_1.add_element(element1, position, split_point - position);
                phrase_2.add_element(element, split_point, position + length - split_point);
            } else {
                phrase_2.add_element(element, position, length);
            }
        }

        (phrase_1, phrase_2)
    }

    pub fn length(&self) -> Fraction {
        let start = self.start();
        let end = self.end();
        end - start
    }

    pub fn min_val(&self) -> u8 {
        let (el, _) = self.elements.values().min_by_key(|(e, _)| e.min()).unwrap();
        el.min()
    }

    pub fn max_val(&self) -> u8 {
        let (el, _) = self.elements.values().max_by_key(|(e, _)| e.max()).unwrap();
        el.max()
    }

    pub fn mean(&self) -> u8 {
        let mut total: usize = 0;
        let mut count: usize = 0;
        for (element, _) in self.elements.values() {
            match element {
                PhraseElement::Note(n) => {
                    total += n.value() as usize;
                    count += 1;
                }
                PhraseElement::Chord(c) => {
                    total += c.iter().fold(0, |acc, n| acc + n.value()) as usize;
                    count += c.len();
                }
            }
        }
        (total / count) as u8
    }

    pub fn mean_at(&self, position: Fraction) -> Option<(u8, u8)> {
        let (pos, (element, len)) = self.elements.range(..=position).rev().next()?;
        if *pos + *len > position {
            return Some(element.mean());
        }
        None
    }

    pub fn min_at(&self, position: Fraction) -> Option<u8> {
        let (pos, (element, len)) = self.elements.range(..=position).rev().next()?;
        if *pos + *len > position {
            return Some(element.min());
        }
        None
    }

    pub fn max_at(&self, position: Fraction) -> Option<u8> {
        let (pos, (element, len)) = self.elements.range(..=position).rev().next()?;
        if *pos + *len > position {
            return Some(element.max());
        }
        None
    }

    pub fn transpose_octaves(&mut self, octaves: i8) {
        for (_, (el, _)) in &mut self.elements {
            el.transpose_octaves(octaves);
        }
    }

    pub fn num_elements(&self) -> usize {
        self.elements.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::fraction::Fraction;
    use crate::phrase::Phrase;
    use crate::phrase_element::{Note, NoteName, PhraseElement, Tie};
    use std::collections::BTreeMap;

    #[test]
    fn phrase_split_start() {
        let mut elements = BTreeMap::new();
        let note = Note::new(NoteName::C, 4, 0, Tie::None);
        elements.insert(
            Fraction::zero(),
            (PhraseElement::Note(note), Fraction::new(2, 1)),
        );
        let phrase = Phrase::new(elements);

        let (split1, split2) = phrase.split(Fraction::zero());

        assert_eq!(split1.num_elements(), 0);
        assert_eq!(split2.num_elements(), 1);

        assert_eq!(split1, Phrase::default());

        let mut split2_expected = BTreeMap::new();
        split2_expected.insert(
            Fraction::zero(),
            (PhraseElement::Note(note), Fraction::new(2, 1)),
        );
        assert_eq!(split2, Phrase::new(split2_expected));
    }

    #[test]
    fn phrase_split_end() {
        let mut elements = BTreeMap::new();
        let note = Note::new(NoteName::C, 4, 0, Tie::None);
        elements.insert(
            Fraction::zero(),
            (PhraseElement::Note(note), Fraction::new(2, 1)),
        );
        let phrase = Phrase::new(elements);

        let (split1, split2) = phrase.split(Fraction::new(2, 1));

        assert_eq!(split1.num_elements(), 1);
        assert_eq!(split2.num_elements(), 0);

        let mut split1_expected = BTreeMap::new();
        split1_expected.insert(
            Fraction::zero(),
            (PhraseElement::Note(note), Fraction::new(2, 1)),
        );
        assert_eq!(split1, Phrase::new(split1_expected));

        assert_eq!(split2, Phrase::default());
    }

    #[test]
    fn phrase_split_middle() {
        let mut elements = BTreeMap::new();
        let note = Note::new(NoteName::C, 4, 0, Tie::None);
        elements.insert(
            Fraction::zero(),
            (PhraseElement::Note(note), Fraction::new(2, 1)),
        );
        let phrase = Phrase::new(elements);

        let (split1, split2) = phrase.split(Fraction::new(1, 1));

        assert_eq!(split1.num_elements(), 1);
        assert_eq!(split2.num_elements(), 1);

        let mut split1_expected = BTreeMap::new();
        let mut note = note.clone();
        note.tie = Tie::Start;
        split1_expected.insert(
            Fraction::zero(),
            (PhraseElement::Note(note), Fraction::new(1, 1)),
        );
        assert_eq!(split1, Phrase::new(split1_expected));

        let mut split2_expected = BTreeMap::new();
        let mut note = note.clone();
        note.tie = Tie::Stop;
        split2_expected.insert(
            Fraction::new(1, 1),
            (PhraseElement::Note(note), Fraction::new(1, 1)),
        );
        assert_eq!(split2, Phrase::new(split2_expected));
    }

    #[test]
    fn phrase_start_end() {
        let mut elements = BTreeMap::new();
        let note = Note::new(NoteName::C, 4, 0, Tie::None);
        elements.insert(
            Fraction::zero(),
            (PhraseElement::Note(note), Fraction::new(2, 1)),
        );
        let phrase = Phrase::new(elements);

        assert_eq!(phrase.start(), Fraction::zero());
        assert_eq!(phrase.end(), Fraction::new(2, 1));
    }
}
