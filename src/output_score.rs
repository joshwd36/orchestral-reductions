use crate::fraction::Fraction;
use crate::music_xml::MusicXML;
use crate::phrase::Phrase;
use crate::phrase_element::*;
use crate::score_representation::*;
use std::collections::BTreeMap;

pub struct OutputScore {
    xml: MusicXML,
}

impl OutputScore {
    /// Convert a StaveList to a MusicXML document.
    pub fn new(stave_list: StaveList) -> Self {
        let num_staves = stave_list.staves.len();
        let mut xml = MusicXML::new();
        let bar_numbers = BarNumbers::new(&stave_list.times);
        let mut phrase_bars: Vec<Vec<(Phrase, u8)>> = Vec::new();
        let mut divisions: Vec<Fraction> = Vec::new();
        for (stave, phrases) in stave_list.staves.into_iter().enumerate() {
            for phrase in phrases {
                let mut current_phrase = phrase;
                while current_phrase.num_elements() > 0 {
                    let start = current_phrase.start();
                    let bar_num = bar_numbers.get_bar_number(start);
                    if bar_num + 1 > phrase_bars.len() {
                        phrase_bars.resize_with(bar_num + 1, Default::default);
                        divisions.resize_with(bar_num + 1, Default::default);
                    }

                    if let Some(split_point) =
                        bar_numbers.crosses_bar(start, current_phrase.length())
                    {
                        let (phrase1, phrase2) = current_phrase.split(split_point);
                        if divisions[bar_num] == Fraction::zero() {
                            divisions[bar_num] = phrase1.min_duration();
                        } else {
                            divisions[bar_num] = divisions[bar_num].min(phrase1.min_duration());
                        }

                        phrase_bars[bar_num].push((phrase1, (stave + 1) as u8));
                        current_phrase = phrase2;
                    } else {
                        if divisions[bar_num] == Fraction::zero() {
                            divisions[bar_num] = current_phrase.min_duration();
                        } else {
                            divisions[bar_num] =
                                divisions[bar_num].min(current_phrase.min_duration());
                        }
                        phrase_bars[bar_num].push((current_phrase, (stave + 1) as u8));
                        current_phrase = Phrase::default();
                    }
                }
            }
        }

        let mut current_pos = Fraction::zero();
        let mut current_time = *stave_list.times.get(&Fraction::zero()).unwrap();
        for (bar_num, mut bar) in phrase_bars.into_iter().enumerate() {
            let smallest = NoteType::from_fraction(divisions[bar_num]);
            let key = stave_list.keys.get(&current_pos).cloned();
            let time = stave_list.times.get(&current_pos).cloned();
            let clefs = if bar_num == 0 {
                if num_staves == 1 {
                    vec![Some(Clef::Treble)]
                } else {
                    let mut clefs = vec![Some(Clef::Treble); num_staves - 1];
                    clefs.push(Some(Clef::Bass));
                    clefs
                }
            } else {
                Vec::new()
            };
            if let Some(time) = time {
                current_time = time;
            }
            let bar_end =
                Fraction::new(4 * current_time.0 as i32, current_time.1 as i32) + current_pos;
            xml.start_bar(
                smallest.into_iter().min().unwrap_or(NoteType::Quarter),
                clefs,
                key,
                time,
            );
            bar.sort_unstable_by_key(|(_, a)| *a);
            let mut voice = 1;
            let mut last_stave = 0;
            for (phrase, stave) in bar {
                if stave > last_stave {
                    voice = (stave - 1) * 4 + 1;
                } else {
                    voice += 1;
                }

                if phrase.num_elements() == 0 {
                    xml.add_rest(NoteType::Whole, voice, stave, true);
                } else {
                    for (start, (element, length)) in phrase.elements() {
                        if start > current_pos {
                            let rests = NoteType::from_fraction(start - current_pos);
                            for rest in rests {
                                xml.add_rest(rest, voice, stave, false);
                                current_pos += rest.get_value();
                            }
                        } else if start < current_pos {
                            let backups = NoteType::from_fraction(current_pos - start);
                            for backup in backups {
                                xml.backup(backup);
                                current_pos -= backup.get_value();
                            }
                        }
                        let lengths = NoteType::from_fraction(length);
                        let num_notes = lengths.len();
                        for (i, length) in lengths.into_iter().enumerate() {
                            let mut element = element.clone();
                            if num_notes > 1 {
                                if i == 0 {
                                    element.start_tie();
                                } else if i == num_notes - 1 {
                                    element.stop_tie();
                                } else {
                                    element.stop_tie();
                                    element.start_tie();
                                }
                            }
                            match element {
                                PhraseElement::Note(note) => {
                                    xml.add_note(length, note, voice, stave, false)
                                }
                                PhraseElement::Chord(ref chord) => match chord.as_slice() {
                                    [] => (),
                                    [x, xs @ ..] => {
                                        xml.add_note(length, *x, voice, stave, false);
                                        for note in xs {
                                            xml.add_note(length, *note, voice, stave, true);
                                        }
                                    }
                                },
                            }
                        }
                        current_pos += length;
                    }
                }

                if current_pos < bar_end {
                    let rests = NoteType::from_fraction(bar_end - current_pos);
                    for rest in rests {
                        xml.add_rest(rest, voice, stave, false);
                        current_pos += rest.get_value();
                    }
                }

                last_stave = stave;
            }
            xml.end_bar()
        }

        xml.end();

        OutputScore { xml }
    }

    /// Get the XML bytes.
    pub fn get_value(self) -> Vec<u8> {
        self.xml.get_value()
    }
}

/// Used for calculating bar numbers based off time signatures.
struct BarNumbers {
    offsets: BTreeMap<Fraction, (usize, (u8, u8))>,
}

impl BarNumbers {
    /// Parse the time signatures and calculate the difference between time signatures.
    pub fn new(times: &BTreeMap<Fraction, (u8, u8)>) -> Self {
        let mut offsets = BTreeMap::new();
        let mut current_pos = Fraction::zero();
        let mut current_time = (1, 1);
        for (start, time) in times {
            let difference = *start - current_pos;
            let num_bars = BarNumbers::num_bars(difference, current_time);
            offsets.insert(*start, (num_bars, *(time)));
            current_pos = *start;
            current_time = *time;
        }

        Self { offsets }
    }

    /// Calculate the number of bars between two positions based on the time signature.
    fn num_bars(time_between: Fraction, time_sig: (u8, u8)) -> usize {
        let time = time_between * Fraction::new(1, 4)
            / Fraction::new(time_sig.0 as i32, time_sig.1 as i32);
        time.to_whole() as usize
    }

    /// Get the bar number at a given position.
    pub fn get_bar_number(&self, time: Fraction) -> usize {
        let (position, (current, time_sig)) = self.offsets.range(..=time).rev().next().unwrap();
        current + Self::num_bars(time - *position, *time_sig)
    }

    /// Check whether a passage crosses a barline given its start position and length, and return the position if it does.
    pub fn crosses_bar(&self, start: Fraction, length: Fraction) -> Option<Fraction> {
        let (&position, (_, time_sig)) = self.offsets.range(..=start).rev().next().unwrap();
        let offset = start - position;
        let time_frac = Fraction::new((time_sig.0 * 4) as i32, time_sig.1 as i32);
        let next_bar = offset - (offset % time_frac) + time_frac;
        if next_bar < offset + length {
            Some(next_bar + position)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fraction::Fraction;
    use crate::output_score::BarNumbers;
    use std::collections::BTreeMap;

    fn setup() -> BarNumbers {
        let mut times = BTreeMap::new();
        times.insert(Fraction::zero(), (4, 4));
        times.insert(Fraction::new(16, 1), (3, 4));
        BarNumbers::new(&times)
    }

    #[test]
    fn bar_numbers() {
        let bars = setup();

        assert_eq!(bars.get_bar_number(Fraction::zero()) + 1, 1);
        assert_eq!(bars.get_bar_number(Fraction::new(4, 1)) + 1, 2);
        assert_eq!(bars.get_bar_number(Fraction::new(16, 1)) + 1, 5);
        assert_eq!(bars.get_bar_number(Fraction::new(19, 1)) + 1, 6);
    }

    #[test]
    fn crosses_bar() {
        let bars = setup();

        assert_eq!(
            bars.crosses_bar(Fraction::zero(), Fraction::new(1, 1)),
            None
        );
        assert_eq!(
            bars.crosses_bar(Fraction::zero(), Fraction::new(4, 1)),
            None
        );
        assert_eq!(
            bars.crosses_bar(Fraction::zero(), Fraction::new(5, 1)),
            Some(Fraction::new(4, 1))
        );
        assert_eq!(
            bars.crosses_bar(Fraction::new(16, 1), Fraction::new(4, 1)),
            Some(Fraction::new(19, 1))
        );
    }
}
