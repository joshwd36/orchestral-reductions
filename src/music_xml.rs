use crate::phrase_element::*;
use quick_xml::events::attributes::Attribute;
use quick_xml::{events::*, Writer};

pub struct MusicXML {
    xml: Writer<Vec<u8>>,
    current_divisions: u32,
    current_bar: u32,
}

impl MusicXML {
    /// Get the byte buffer back.
    pub fn get_value(self) -> Vec<u8> {
        self.xml.into_inner()
    }

    /// Create a new MusicXML document and populate with a piano part.
    pub fn new() -> MusicXML {
        let mut xml = Writer::new_with_indent(Vec::new(), b' ', 4);
        //let mut xml = Writer::new(Vec::new());
        xml.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))
            .unwrap();
        xml.write_event(Event::DocType(BytesText::from_escaped_str(" score-partwise PUBLIC \"-//Recordare//DTD MusicXML 3.1 Partwise//EN\" \"http://www.musicxml.org/dtds/partwise.dtd\""))).unwrap();
        let mut score_partwise = BytesStart::owned_name("score-partwise");
        score_partwise.push_attribute(("version", "3.1"));
        xml.write_event(Event::Start(score_partwise)).unwrap();
        xml.write_event(Event::Start(BytesStart::owned_name("part-list")))
            .unwrap();
        let mut score_part = BytesStart::owned_name("score-part");
        score_part.push_attribute(("id", "P1"));
        xml.write_event(Event::Start(score_part)).unwrap();
        xml.write_event(Event::Start(BytesStart::owned_name("part-name")))
            .unwrap();
        xml.write_event(Event::Text(BytesText::from_escaped_str("Piano")))
            .unwrap();
        xml.write_event(Event::End(BytesEnd::borrowed(b"part-name")))
            .unwrap();
        xml.write_event(Event::End(BytesEnd::borrowed(b"score-part")))
            .unwrap();
        xml.write_event(Event::End(BytesEnd::borrowed(b"part-list")))
            .unwrap();
        let mut part = BytesStart::owned_name("part");
        part.push_attribute(("id", "P1"));
        xml.write_event(Event::Start(part)).unwrap();
        MusicXML {
            xml,
            current_divisions: 0,
            current_bar: 1,
        }
    }

    /// End a MusicXML document.
    pub fn end(&mut self) {
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"part")))
            .unwrap();
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"score-partwise")))
            .unwrap();
    }

    /// Start the bar and write the attributes.
    pub fn start_bar(
        &mut self,
        divisions: NoteType,
        clefs: Vec<Option<Clef>>,
        key: Option<i8>,
        time: Option<(u8, u8)>,
    ) {
        self.current_divisions = divisions.to_divisions();
        let mut measure = BytesStart::owned_name("measure");
        measure.push_attribute(("number", self.current_bar.to_string().as_str()));
        self.xml.write_event(Event::Start(measure)).unwrap();
        self.xml
            .write_event(Event::Start(BytesStart::owned_name("attributes")))
            .unwrap();

        self.xml
            .write_event(Event::Start(BytesStart::owned_name("divisions")))
            .unwrap();
        self.xml
            .write_event(Event::Text(BytesText::from_plain_str(
                &self.current_divisions.to_string(),
            )))
            .unwrap();
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"divisions")))
            .unwrap();

        if let Some(fifths) = key {
            self.xml
                .write_event(Event::Start(BytesStart::owned_name("key")))
                .unwrap();

            self.xml
                .write_event(Event::Start(BytesStart::owned_name("fifths")))
                .unwrap();
            self.xml
                .write_event(Event::Text(BytesText::from_plain_str(&fifths.to_string())))
                .unwrap();
            self.xml
                .write_event(Event::End(BytesEnd::borrowed(b"fifths")))
                .unwrap();

            self.xml
                .write_event(Event::End(BytesEnd::borrowed(b"key")))
                .unwrap();
        }
        if let Some((beats, beat_type)) = time {
            self.xml
                .write_event(Event::Start(BytesStart::owned_name("time")))
                .unwrap();

            self.xml
                .write_event(Event::Start(BytesStart::owned_name("beats")))
                .unwrap();
            self.xml
                .write_event(Event::Text(BytesText::from_plain_str(&beats.to_string())))
                .unwrap();
            self.xml
                .write_event(Event::End(BytesEnd::borrowed(b"beats")))
                .unwrap();

            self.xml
                .write_event(Event::Start(BytesStart::owned_name("beat-type")))
                .unwrap();
            self.xml
                .write_event(Event::Text(BytesText::from_plain_str(
                    &beat_type.to_string(),
                )))
                .unwrap();
            self.xml
                .write_event(Event::End(BytesEnd::borrowed(b"beat-type")))
                .unwrap();

            self.xml
                .write_event(Event::End(BytesEnd::borrowed(b"time")))
                .unwrap();
        }
        for (number, clef) in clefs.into_iter().enumerate() {
            if let Some(clef) = clef {
                let mut clef_element = BytesStart::owned_name("clef");
                clef_element.push_attribute(Attribute::from((
                    "number",
                    (number + 1).to_string().as_str(),
                )));
                self.xml.write_event(Event::Start(clef_element)).unwrap();

                self.xml
                    .write_event(Event::Start(BytesStart::owned_name("sign")))
                    .unwrap();
                self.xml
                    .write_event(Event::Text(BytesText::from_plain_str(clef.sign())))
                    .unwrap();
                self.xml
                    .write_event(Event::End(BytesEnd::borrowed(b"sign")))
                    .unwrap();

                self.xml
                    .write_event(Event::Start(BytesStart::owned_name("line")))
                    .unwrap();
                self.xml
                    .write_event(Event::Text(BytesText::from_plain_str(clef.line())))
                    .unwrap();
                self.xml
                    .write_event(Event::End(BytesEnd::borrowed(b"line")))
                    .unwrap();

                self.xml
                    .write_event(Event::End(BytesEnd::borrowed(b"clef")))
                    .unwrap();
            }
        }
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"attributes")))
            .unwrap();
    }

    /// End the bar.
    pub fn end_bar(&mut self) {
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"measure")))
            .unwrap();
        self.current_bar += 1;
    }

    /// Used to write elements common to notes and rests.
    fn write_note_common(&mut self, length: NoteType, voice: u8, stave: u8, tie: Tie) {
        self.xml
            .write_event(Event::Start(BytesStart::owned_name("duration")))
            .unwrap();
        self.xml
            .write_event(Event::Text(BytesText::from_plain_str(
                &length.divisions(self.current_divisions).to_string(),
            )))
            .unwrap();
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"duration")))
            .unwrap();

        match tie {
            Tie::Start => {
                let mut tie = BytesStart::owned_name("tie");
                tie.push_attribute(("type", "start"));
                self.xml.write_event(Event::Empty(tie)).unwrap();
            }
            Tie::Stop => {
                let mut tie = BytesStart::owned_name("tie");
                tie.push_attribute(("type", "stop"));
                self.xml.write_event(Event::Empty(tie)).unwrap();
            }
            Tie::StartStop => {
                let mut tie = BytesStart::owned_name("tie");
                tie.push_attribute(("type", "stop"));
                self.xml.write_event(Event::Empty(tie)).unwrap();

                let mut tie = BytesStart::owned_name("tie");
                tie.push_attribute(("type", "start"));
                self.xml.write_event(Event::Empty(tie)).unwrap();
            }
            Tie::None => (),
        };

        self.xml
            .write_event(Event::Start(BytesStart::owned_name("voice")))
            .unwrap();
        self.xml
            .write_event(Event::Text(BytesText::from_plain_str(&voice.to_string())))
            .unwrap();
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"voice")))
            .unwrap();

        self.xml
            .write_event(Event::Start(BytesStart::owned_name("type")))
            .unwrap();
        self.xml
            .write_event(Event::Text(BytesText::from_plain_str(
                &length.name().to_string(),
            )))
            .unwrap();
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"type")))
            .unwrap();

        self.xml
            .write_event(Event::Start(BytesStart::owned_name("staff")))
            .unwrap();
        self.xml
            .write_event(Event::Text(BytesText::from_plain_str(&stave.to_string())))
            .unwrap();
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"staff")))
            .unwrap();

        self.xml
            .write_event(Event::Start(BytesStart::owned_name("notations")))
            .unwrap();
        match tie {
            Tie::Start => {
                let mut tie = BytesStart::owned_name("tied");
                tie.push_attribute(("type", "start"));
                self.xml.write_event(Event::Empty(tie)).unwrap();
            }
            Tie::Stop => {
                let mut tie = BytesStart::owned_name("tied");
                tie.push_attribute(("type", "stop"));
                self.xml.write_event(Event::Empty(tie)).unwrap();
            }
            Tie::StartStop => {
                let mut tie = BytesStart::owned_name("tied");
                tie.push_attribute(("type", "stop"));
                self.xml.write_event(Event::Empty(tie)).unwrap();

                let mut tie = BytesStart::owned_name("tied");
                tie.push_attribute(("type", "start"));
                self.xml.write_event(Event::Empty(tie)).unwrap();
            }
            Tie::None => (),
        };
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"notations")))
            .unwrap();
    }

    /// Add a rest to a bar.
    pub fn add_rest(&mut self, length: NoteType, voice: u8, stave: u8, bar_rest: bool) {
        self.xml
            .write_event(Event::Start(BytesStart::owned_name("note")))
            .unwrap();
        let mut rest = BytesStart::owned_name("rest");
        if bar_rest {
            rest.push_attribute(("measure", "yes"));
        }
        self.xml.write_event(Event::Empty(rest)).unwrap();
        self.write_note_common(length, voice, stave, Tie::None);
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"note")))
            .unwrap();
    }

    /// Add a note to a bar.
    pub fn add_note(&mut self, length: NoteType, note: Note, voice: u8, stave: u8, chord: bool) {
        self.xml
            .write_event(Event::Start(BytesStart::owned_name("note")))
            .unwrap();

        if chord {
            self.xml
                .write_event(Event::Empty(BytesStart::owned_name("chord")))
                .unwrap();
        }

        self.xml
            .write_event(Event::Start(BytesStart::owned_name("pitch")))
            .unwrap();

        self.xml
            .write_event(Event::Start(BytesStart::owned_name("step")))
            .unwrap();
        self.xml
            .write_event(Event::Text(BytesText::from_plain_str(note.step.name())))
            .unwrap();
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"step")))
            .unwrap();
        if note.alter != 0 {
            self.xml
                .write_event(Event::Start(BytesStart::owned_name("alter")))
                .unwrap();
            self.xml
                .write_event(Event::Text(BytesText::from_plain_str(
                    &note.alter.to_string(),
                )))
                .unwrap();
            self.xml
                .write_event(Event::End(BytesEnd::borrowed(b"alter")))
                .unwrap();
        }

        self.xml
            .write_event(Event::Start(BytesStart::owned_name("octave")))
            .unwrap();
        self.xml
            .write_event(Event::Text(BytesText::from_plain_str(
                &note.octave.to_string(),
            )))
            .unwrap();
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"octave")))
            .unwrap();
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"pitch")))
            .unwrap();

        self.write_note_common(length, voice, stave, note.tie);

        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"note")))
            .unwrap();
    }

    /// Add a backup element to a bar.
    pub fn backup(&mut self, time: NoteType) {
        self.xml
            .write_event(Event::Start(BytesStart::owned_name("backup")))
            .unwrap();
        self.xml
            .write_event(Event::Start(BytesStart::owned_name("duration")))
            .unwrap();
        self.xml
            .write_event(Event::Text(BytesText::from_plain_str(
                &time.divisions(self.current_divisions).to_string(),
            )))
            .unwrap();
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"duration")))
            .unwrap();
        self.xml
            .write_event(Event::End(BytesEnd::borrowed(b"backup")))
            .unwrap();
    }
}
