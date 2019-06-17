//! A simple [https://en.wikipedia.org/wiki/RIS_(file_format)](RIS bibliography file) (de)serializer for Rust.
use std::{
    convert::Infallible,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use lazy_static::lazy_static;
use regex::Regex;

/// A RIS reference list
///
/// A RIS file has no information other than the sequence of its entries, so this type is just a wrapper around `Vec<Entry>`,
/// with associated functions for (de)serialization.
///
/// This type implements [std::fmt::Display] and [std::str::FromStr] to (de)serialize to/from strings.
///
/// See [crate::Entry] for more information.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RIS(Vec<Entry>);

impl FromStr for RIS {
    type Err = ParseError;

    /// Parse a RIS file from a string.
    /// See [Entry](crate::Entry) for more information on how keys are mapped to fields.
    fn from_str(s: &str) -> Result<RIS, Self::Err> {
        use ParseErrorKind::*;
        use ReferenceType::*;

        let mut entries = Vec::new();
        let mut current_entry: Option<CurrentEntry> = None;

        for (line_no, line) in s.lines().enumerate() {
            let line_no = line_no + 1;

            lazy_static! {
                static ref LINE_RE: Regex = Regex::new("([A-Z][A-Z0-9])  - (.*)").unwrap();
            }

            let matches = LINE_RE
                .captures(line)
                .ok_or_else(|| ParseError::new(line_no, InvalidLine))?;

            let key = matches.get(1).unwrap().as_str();
            let value = matches.get(2).unwrap().as_str();

            match current_entry.as_mut() {
                Some(CurrentEntry { entry, .. }) => match key {
                    "TY" => return Err(ParseError::new(line_no, UnterminatedEntry)),

                    "ID" => set_unique_field(&mut entry.id, String::from(value), line_no)?,

                    "T1" | "TI" => {
                        set_unique_field(&mut entry.title, String::from(value), line_no)?
                    }
                    "T2" => {
                        set_unique_field(&mut entry.secondary_title, String::from(value), line_no)?
                    }
                    "T3" => {
                        set_unique_field(&mut entry.tertiary_title, String::from(value), line_no)?
                    }

                    "A1" | "AU" => entry.authors.push(String::from(value)),
                    "A2" | "ED" => entry.secondary_authors.push(String::from(value)),
                    "A3" => entry.tertiary_authors.push(String::from(value)),

                    "Y1" | "PY" => set_unique_field(
                        &mut entry.primary_date,
                        PublicationDate::parse(value, line_no)?,
                        line_no,
                    )?,
                    "Y2" => set_unique_field(
                        &mut entry.secondary_date,
                        PublicationDate::parse(value, line_no)?,
                        line_no,
                    )?,

                    "N1" => set_unique_field(&mut entry.notes, String::from(value), line_no)?,

                    "AB" | "N2" => {
                        set_unique_field(&mut entry.abstract_, String::from(value), line_no)?
                    }
                    "KW" => entry.keywords.push(String::from(value)),
                    "RP" => set_unique_field(&mut entry.reprint, String::from(value), line_no)?,
                    "AV" => {
                        set_unique_field(&mut entry.availability, String::from(value), line_no)?
                    }

                    "SP" => set_unique_field(&mut entry.start_page, String::from(value), line_no)?,
                    "EP" => set_unique_field(&mut entry.end_page, String::from(value), line_no)?,

                    "JF" | "JO" => {
                        set_unique_field(&mut entry.journal, String::from(value), line_no)?
                    }
                    "JA" => {
                        set_unique_field(&mut entry.journal_abbrev, String::from(value), line_no)?
                    }
                    "J1" => {
                        set_unique_field(&mut entry.journal_abbrev_1, String::from(value), line_no)?
                    }
                    "J2" => {
                        set_unique_field(&mut entry.journal_abbrev_2, String::from(value), line_no)?
                    }

                    "VL" => set_unique_field(&mut entry.volume, String::from(value), line_no)?,
                    "IS" => set_unique_field(&mut entry.issue, String::from(value), line_no)?,
                    "CY" => set_unique_field(&mut entry.city, String::from(value), line_no)?,
                    "PB" => set_unique_field(&mut entry.publisher, String::from(value), line_no)?,
                    "SN" => {
                        set_unique_field(&mut entry.serial_number, String::from(value), line_no)?
                    }
                    "AD" => set_unique_field(&mut entry.address, String::from(value), line_no)?,

                    "U1" => set_unique_field(&mut entry.user_1, String::from(value), line_no)?,
                    "U2" => set_unique_field(&mut entry.user_2, String::from(value), line_no)?,
                    "U3" => set_unique_field(&mut entry.user_3, String::from(value), line_no)?,
                    "U4" => set_unique_field(&mut entry.user_4, String::from(value), line_no)?,
                    "U5" => set_unique_field(&mut entry.user_5, String::from(value), line_no)?,

                    "M1" => set_unique_field(&mut entry.misc_1, String::from(value), line_no)?,
                    "M2" => set_unique_field(&mut entry.misc_2, String::from(value), line_no)?,
                    "M3" => set_unique_field(&mut entry.misc_3, String::from(value), line_no)?,

                    "BT" => {
                        let field = match entry.reference_type {
                            WholeBook | UnpublishedWork => &mut entry.title,
                            _ => &mut entry.secondary_title,
                        };

                        set_unique_field(field, String::from(value), line_no)?;
                    }

                    "ER" => {
                        if value.is_empty() {
                            entries.push(current_entry.take().unwrap().entry);
                        } else {
                            return Err(ParseError::new(line_no, InvalidLine));
                        }
                    }

                    _ => {
                        return Err(ParseError::new(line_no, InvalidKey));
                    }
                },
                None => match key {
                    "TY" => {
                        current_entry = Some(CurrentEntry {
                            entry: Entry::new(value.parse().unwrap()),
                            start_line_no: line_no,
                        })
                    }

                    _ => return Err(ParseError::new(line_no, InvalidKey))?,
                },
            }
        }

        if let Some(current) = current_entry {
            Err(ParseError::new(current.start_line_no, UnterminatedEntry))
        } else {
            Ok(RIS(entries))
        }
    }
}

impl Display for RIS {
    /// Serializes a slice of entries into a multi-entry RIS string
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(entry) = self.0.first() {
            entry.fmt(f)?;

            for entry in self.0.iter().skip(1) {
                writeln!(f)?;
                entry.fmt(f)?;
            }
        }

        Ok(())
    }
}

/// A single entry in the RIS file, started by a `TY` and terminated by a `ER`.
///
/// This type implements [std::fmt::Display] and [std::str::FromStr] to (de)serialize to/from strings.
/// To (de)serialize files composed of multiple records, see [crate::RIS]
///
/// # Field mappings
///
/// All fields except `TY` are optional, and are stored as `Option`s.
/// Repeated fields are invalid, and will cause a [crate::ParseError].
///
/// [ReferenceType]: crate::ReferenceType
/// [String]: std::string::String
/// [PublicationDate]: crate::PublicationDate
///
/// | Key  | Field              | Type              |
/// |------|--------------------|-------------------|
/// | `TY` | `reference_type`   | [ReferenceType]   |
/// | `ID` | `id`               | [String]          |
/// | `T1` | `title`            | [String]          |
/// | `T2` | `secondary_title`  | [String]          |
/// | `T3` | `tertiary_title`   | [String]          |
/// | `Y1` | `primary_date`     | [PublicationDate] |
/// | `Y2` | `secondary_date`   | [PublicationDate] |
/// | `N1` | `notes`            | [String]          |
/// | `N2` | `abstract_`        | [String]          |
/// | `RP` | `reprint`          | [String]          |
/// | `AV` | `availability`     | [String]          |
/// | `SP` | `start_page`       | [String]          |
/// | `EP` | `end_page`         | [String]          |
/// | `JA` | `journal_abbrev`   | [String]          |
/// | `J1` | `journal_abbrev_1` | [String]          |
/// | `J2` | `journal_abbrev_2` | [String]          |
/// | `VL` | `volume`           | [String]          |
/// | `IS` | `issue`            | [String]          |
/// | `CY` | `city`             | [String]          |
/// | `PB` | `publisher`        | [String]          |
/// | `SN` | `serial_number`    | [String]          |
/// | `AD` | `address`          | [String]          |
/// | `U1` | `user_1`           | [String]          |
/// | `U2` | `user_2`           | [String]          |
/// | `U3` | `user_3`           | [String]          |
/// | `U4` | `user_4`           | [String]          |
/// | `U5` | `user_5`           | [String]          |
/// | `M1` | `misc_1`           | [String]          |
/// | `M2` | `misc_2`           | [String]          |
/// | `M3` | `misc_3`           | [String]          |
///
/// Some fields are `Vec`s, and the corresponding keys are allowed to appear multiple times:
///
/// | Key  | Field              | Type     |
/// |------|--------------------|----------|
/// | `A1` | `authors`          | [String] |
/// | `A2` | `second_authors`   | [String] |
/// | `A3` | `tertiary_authors` | [String] |
/// | `KW` | `keywords`         | [String] |
///
/// # Field oddities
///
/// **Note that the following behaviours are inconsistently documented and I am by no means a bibliography expert.**
/// **I wasn't able to find authoritative, conclusive documentation on how to handle them in a standard manner, so this is my best attempt at it.**
/// **Any help hunting down information would be appreciated.**
///
/// During parsing, some keys are considered synonims and mapped to a common field:
///
/// | Key  | Synonims   | Field             |
/// |------|------------|-------------------|
/// | `T1` | `TI`       | `title`           |
/// | `A1` | `AU`       | `first_authors`   |
/// | `A2` | `ED`       | `second_authors`  |
/// | `T2` | `JF`, `JO` | `secondary_title` |
/// | `Y1` | `PY`       | `primary_date`    |
/// | `N2` | `AB`       | `abstract_`       |
///
/// Some synonims are mapped conditionally depending on the reference type `TY`:
///
/// | `TY`               | Key  | Synonims | Field             |
/// |--------------------|------|----------|-------------------|
/// | `Whole Book`       | `T1` | `BT`     | `title`           |
/// | `Unpublished Work` | `T2` | `BT`     | `secondary_title` |
/// | `Unpublished Work` | `T1` | `CT`     | `title`           |
///
/// Some bibliography systems may resolve a journal abbreviation (`JA/J2`) as a standard abbreviated name for a journal, and automatically populate `T2` with the full journal name.
/// This behaviour is not implemented as I could only find inconsistent documentation for it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entry {
    pub reference_type: ReferenceType, // TY

    pub id: Option<String>, // ID

    pub title: Option<String>,           // T1, TI
    pub secondary_title: Option<String>, // T2
    pub tertiary_title: Option<String>,  // T3

    pub authors: Vec<String>,           // AU, A1
    pub secondary_authors: Vec<String>, // A2, ED
    pub tertiary_authors: Vec<String>,  // A3

    pub primary_date: Option<PublicationDate>,   // PY, Y1
    pub secondary_date: Option<PublicationDate>, // Y2

    pub notes: Option<String>, // N1

    pub abstract_: Option<String>,    // AB, N2
    pub keywords: Vec<String>,        // KW
    pub reprint: Option<String>,      // RP
    pub availability: Option<String>, // AV

    pub start_page: Option<String>, // SP
    pub end_page: Option<String>,   // EP

    pub journal: Option<String>,          // JF, JO
    pub journal_abbrev: Option<String>,   // JA
    pub journal_abbrev_1: Option<String>, // J1
    pub journal_abbrev_2: Option<String>, // J2

    pub volume: Option<String>,        // VL
    pub issue: Option<String>,         // IS
    pub city: Option<String>,          // CY
    pub publisher: Option<String>,     // PB
    pub serial_number: Option<String>, // SN
    pub address: Option<String>,       // AD

    pub user_1: Option<String>, // U1
    pub user_2: Option<String>, // U2
    pub user_3: Option<String>, // U3
    pub user_4: Option<String>, // U4
    pub user_5: Option<String>, // U5

    pub misc_1: Option<String>, // M1
    pub misc_2: Option<String>, // M2
    pub misc_3: Option<String>, // M3
}

impl Entry {
    pub fn new(reference_type: ReferenceType) -> Self {
        Self {
            reference_type,

            id: None,

            title: None,
            secondary_title: None,
            tertiary_title: None,

            authors: Vec::new(),
            secondary_authors: Vec::new(),
            tertiary_authors: Vec::new(),

            primary_date: None,
            secondary_date: None,

            notes: None,

            abstract_: None,
            keywords: Vec::new(),
            reprint: None,
            availability: None,

            start_page: None,
            end_page: None,

            journal_abbrev: None,
            journal: None,
            journal_abbrev_1: None,
            journal_abbrev_2: None,

            volume: None,
            issue: None,
            city: None,
            publisher: None,
            serial_number: None,
            address: None,

            user_1: None,
            user_2: None,
            user_3: None,
            user_4: None,
            user_5: None,

            misc_1: None,
            misc_2: None,
            misc_3: None,
        }
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "TY  - {}", &self.reference_type)?;

        write_tag(f, "ID", &self.id)?;

        write_tag(f, "T1", &self.title)?;
        write_tag(f, "T2", &self.secondary_title)?;
        write_tag(f, "T3", &self.tertiary_title)?;

        write_tags(f, "A1", &self.authors)?;
        write_tags(f, "A2", &self.secondary_authors)?;
        write_tags(f, "A3", &self.tertiary_authors)?;

        write_tag(f, "Y1", &self.primary_date)?;
        write_tag(f, "Y2", &self.secondary_date)?;

        write_tag(f, "N1", &self.notes)?;
        write_tag(f, "AB", &self.abstract_)?;

        write_tags(f, "KW", &self.keywords)?;

        write_tag(f, "RP", &self.reprint)?;
        write_tag(f, "AV", &self.availability)?;

        write_tag(f, "SP", &self.start_page)?;
        write_tag(f, "EP", &self.end_page)?;

        write_tag(f, "JF", &self.journal)?;
        write_tag(f, "JA", &self.journal_abbrev)?;
        write_tag(f, "J1", &self.journal_abbrev_1)?;
        write_tag(f, "J2", &self.journal_abbrev_2)?;

        write_tag(f, "VL", &self.volume)?;
        write_tag(f, "IS", &self.issue)?;
        write_tag(f, "CY", &self.city)?;
        write_tag(f, "PB", &self.publisher)?;
        write_tag(f, "SN", &self.serial_number)?;
        write_tag(f, "AD", &self.address)?;

        write_tag(f, "U1", &self.user_1)?;
        write_tag(f, "U2", &self.user_2)?;
        write_tag(f, "U3", &self.user_3)?;
        write_tag(f, "U4", &self.user_4)?;
        write_tag(f, "U5", &self.user_5)?;

        write_tag(f, "M1", &self.misc_1)?;
        write_tag(f, "M2", &self.misc_2)?;
        write_tag(f, "M3", &self.misc_3)?;

        write!(f, "ER  - ")?;

        Ok(())
    }
}

#[inline(always)]
fn write_tag<T: Display>(f: &mut Formatter, tag: &str, field: &Option<T>) -> fmt::Result {
    if let Some(ref value) = field {
        writeln!(f, "{}  - {}", tag, value)?;
    }

    Ok(())
}

#[inline(always)]
fn write_tags<T: Display>(f: &mut Formatter, tag: &str, field: &[T]) -> fmt::Result {
    for value in field.iter() {
        writeln!(f, "{}  - {}", tag, value)?;
    }

    Ok(())
}

/// The type of a reference.
///
/// This type implements [std::fmt::Display] and [std::str::FromStr] to (de)serialize to/from strings.
///
/// # Abbreviations
///
/// This enum encodes standard abbreviations in its variants according to the table below.
/// If the type of a reference doesn't match any of the below abbreviations, it is encoded in the `Other` variant.
///
/// | Abbreviation | Variant                 |
/// |--------------|-------------------------|
/// | ABST         | `Abstract`              |
/// | ADVS         | `AudiovisualMaterial`   |
/// | AGGR         | `AggregatedDatabase`    |
/// | ANCIENT      | `AncientText`           |
/// | ART          | `ArtWork`               |
/// | BILL         | `Bill`                  |
/// | BLOG         | `Blog`                  |
/// | BOOK         | `WholeBook`             |
/// | CASE         | `Case`                  |
/// | CHAP         | `BookChapter`           |
/// | CHART        | `Chart`                 |
/// | CLSWK        | `ClassicalWork`         |
/// | COMP         | `ComputerProgram`       |
/// | CONF         | `ConferenceProceeding`  |
/// | CPAPER       | `ConferencePaper`       |
/// | CTLG         | `Catalog`               |
/// | DATA         | `DataFile`              |
/// | DBASE        | `OnlineDatabase`        |
/// | DICT         | `Dictionary`            |
/// | EBOOK        | `ElectronicBook`        |
/// | ECHAP        | `ElectronicBookSection` |
/// | EDBOOK       | `EditedBook`            |
/// | EJOUR        | `ElectronicArticle`     |
/// | ELEC         | `WebPage`               |
/// | ENCYC        | `Encyclopedia`          |
/// | EQUA         | `Equation`              |
/// | FIGURE       | `Figure`                |
/// | GEN          | `Generic`               |
/// | GOVDOC       | `GovernmentDocument`    |
/// | GRANT        | `Grant`                 |
/// | HEAR         | `Hearing`               |
/// | ICOMM        | `InternetCommunication` |
/// | INPR         | `InPress`               |
/// | JFULL        | `JournalFull`           |
/// | JOUR         | `Journal`               |
/// | LEGAL        | `LegalRuleOrRegulation` |
/// | MANSCPT      | `Manuscript`            |
/// | MAP          | `Map`                   |
/// | MGZN         | `MagazineArticle`       |
/// | MPCT         | `MotionPicture`         |
/// | MULTI        | `OnlineMultimedia`      |
/// | MUSIC        | `MusicScore`            |
/// | NEWS         | `Newspaper`             |
/// | PAMP         | `Pamphlet`              |
/// | PAT          | `Patent`                |
/// | PCOMM        | `PersonalCommunication` |
/// | RPRT         | `Report`                |
/// | SER          | `SerialPublication`     |
/// | SLIDE        | `Slide`                 |
/// | SOUND        | `SoundRecording`        |
/// | STAND        | `Standard`              |
/// | STAT         | `Statute`               |
/// | THES         | `ThesisOrDissertation`  |
/// | UNPB         | `UnpublishedWork`       |
/// | VIDEO        | `VideoRecording`        |
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReferenceType {
    Abstract,
    AudiovisualMaterial,
    AggregatedDatabase,
    AncientText,
    ArtWork,
    Bill,
    Blog,
    WholeBook,
    Case,
    BookChapter,
    Chart,
    ClassicalWork,
    ComputerProgram,
    ConferenceProceeding,
    ConferencePaper,
    Catalog,
    DataFile,
    OnlineDatabase,
    Dictionary,
    ElectronicBook,
    ElectronicBookSection,
    EditedBook,
    ElectronicArticle,
    WebPage,
    Encyclopedia,
    Equation,
    Figure,
    Generic,
    GovernmentDocument,
    Grant,
    Hearing,
    InternetCommunication,
    InPress,
    JournalFull,
    Journal,
    LegalRuleOrRegulation,
    Manuscript,
    Map,
    MagazineArticle,
    MotionPicture,
    OnlineMultimedia,
    MusicScore,
    Newspaper,
    Pamphlet,
    Patent,
    PersonalCommunication,
    Report,
    SerialPublication,
    Slide,
    SoundRecording,
    Standard,
    Statute,
    ThesisOrDissertation,
    UnpublishedWork,
    VideoRecording,
    Other(String),
}

impl FromStr for ReferenceType {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use ReferenceType::*;

        Ok(match s {
            "ABST" => Abstract,
            "ADVS" => AudiovisualMaterial,
            "AGGR" => AggregatedDatabase,
            "ANCIENT" => AncientText,
            "ART" => ArtWork,
            "BILL" => Bill,
            "BLOG" => Blog,
            "BOOK" => WholeBook,
            "CASE" => Case,
            "CHAP" => BookChapter,
            "CHART" => Chart,
            "CLSWK" => ClassicalWork,
            "COMP" => ComputerProgram,
            "CONF" => ConferenceProceeding,
            "CPAPER" => ConferencePaper,
            "CTLG" => Catalog,
            "DATA" => DataFile,
            "DBASE" => OnlineDatabase,
            "DICT" => Dictionary,
            "EBOOK" => ElectronicBook,
            "ECHAP" => ElectronicBookSection,
            "EDBOOK" => EditedBook,
            "EJOUR" => ElectronicArticle,
            "ELEC" => WebPage,
            "ENCYC" => Encyclopedia,
            "EQUA" => Equation,
            "FIGURE" => Figure,
            "GEN" => Generic,
            "GOVDOC" => GovernmentDocument,
            "GRANT" => Grant,
            "HEAR" => Hearing,
            "ICOMM" => InternetCommunication,
            "INPR" => InPress,
            "JFULL" => JournalFull,
            "JOUR" => Journal,
            "LEGAL" => LegalRuleOrRegulation,
            "MANSCPT" => Manuscript,
            "MAP" => Map,
            "MGZN" => MagazineArticle,
            "MPCT" => MotionPicture,
            "MULTI" => OnlineMultimedia,
            "MUSIC" => MusicScore,
            "NEWS" => Newspaper,
            "PAMP" => Pamphlet,
            "PAT" => Patent,
            "PCOMM" => PersonalCommunication,
            "RPRT" => Report,
            "SER" => SerialPublication,
            "SLIDE" => Slide,
            "SOUND" => SoundRecording,
            "STAND" => Standard,
            "STAT" => Statute,
            "THES" => ThesisOrDissertation,
            "UNPB" => UnpublishedWork,
            "VIDEO" => VideoRecording,
            _ => Other(s.to_owned()),
        })
    }
}

impl Display for ReferenceType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use ReferenceType::*;

        let s = match self {
            Abstract => "ABST",
            AudiovisualMaterial => "ADVS",
            AggregatedDatabase => "AGGR",
            AncientText => "ANCIENT",
            ArtWork => "ART",
            Bill => "BILL",
            Blog => "BLOG",
            WholeBook => "BOOK",
            Case => "CASE",
            BookChapter => "CHAP",
            Chart => "CHART",
            ClassicalWork => "CLSWK",
            ComputerProgram => "COMP",
            ConferenceProceeding => "CONF",
            ConferencePaper => "CPAPER",
            Catalog => "CTLG",
            DataFile => "DATA",
            OnlineDatabase => "DBASE",
            Dictionary => "DICT",
            ElectronicBook => "EBOOK",
            ElectronicBookSection => "ECHAP",
            EditedBook => "EDBOOK",
            ElectronicArticle => "EJOUR",
            WebPage => "ELEC",
            Encyclopedia => "ENCYC",
            Equation => "EQUA",
            Figure => "FIGURE",
            Generic => "GEN",
            GovernmentDocument => "GOVDOC",
            Grant => "GRANT",
            Hearing => "HEAR",
            InternetCommunication => "ICOMM",
            InPress => "INPR",
            JournalFull => "JFULL",
            Journal => "JOUR",
            LegalRuleOrRegulation => "LEGAL",
            Manuscript => "MANSCPT",
            Map => "MAP",
            MagazineArticle => "MGZN",
            MotionPicture => "MPCT",
            OnlineMultimedia => "MULTI",
            MusicScore => "MUSIC",
            Newspaper => "NEWS",
            Pamphlet => "PAMP",
            Patent => "PAT",
            PersonalCommunication => "PCOMM",
            Report => "RPRT",
            SerialPublication => "SER",
            Slide => "SLIDE",
            SoundRecording => "SOUND",
            Standard => "STAND",
            Statute => "STAT",
            ThesisOrDissertation => "THES",
            UnpublishedWork => "UNPB",
            VideoRecording => "VIDEO",
            Other(s) => &s,
        };

        f.write_str(s)
    }
}

/// The (partial) date of publication of a reference.
///
/// The `year` field is mandatory, all the others are optional.
///
/// This type implements [std::fmt::Display] and [std::str::FromStr] to (de)serialize to/from strings.
///
/// Some examples of valid strings:
/// - `1998///`
/// - `1995/12/01/someotherinfo`
/// - `1998/03//`
/// - `1998///someotherinfo`
/// - `2001`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublicationDate {
    pub year: i32,
    pub month: Option<i32>,
    pub day: Option<i32>,
    pub other_info: Option<String>,
}

impl PublicationDate {
    pub fn new(
        year: i32,
        month: Option<i32>,
        day: Option<i32>,
        other_info: Option<String>,
    ) -> Self {
        Self {
            year,
            month,
            day,
            other_info,
        }
    }

    fn parse(s: &str, line_no: usize) -> Result<Self, ParseError> {
        Self::from_str(s).map_err(|_| ParseError::new(line_no, ParseErrorKind::InvalidDate))
    }
}

pub struct ParseDateError;

impl FromStr for PublicationDate {
    type Err = ParseDateError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        lazy_static! {
            static ref DATE_RE: Regex =
                Regex::new("(\\d\\d\\d\\d)(?:/(\\d\\d)?(?:/(\\d\\d)?(?:/(.+)?)?)?)?").unwrap();
        }

        let matches = DATE_RE.captures(s).ok_or(ParseDateError)?;

        let year = matches
            .get(1)
            .unwrap()
            .as_str()
            .parse()
            .map_err(|_| ParseDateError)?;

        let month = matches
            .get(2)
            .map(|m| m.as_str().parse())
            .transpose()
            .map_err(|_| ParseDateError)?;

        let day = matches
            .get(3)
            .map(|d| d.as_str().parse())
            .transpose()
            .map_err(|_| ParseDateError)?;

        let other_info = matches.get(4).map(|s| s.as_str().to_owned());

        Ok(Self::new(year, month, day, other_info))
    }
}

impl Display for PublicationDate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:04}/", self.year)?;

        if let Some(month) = self.month {
            write!(f, "{:02}", month)?;
        }

        write!(f, "/")?;

        if let Some(day) = self.day {
            write!(f, "{:02}", day)?;
        }

        write!(f, "/")?;

        if let Some(ref other_info) = self.other_info {
            write!(f, "{}", other_info)?;
        }

        Ok(())
    }
}

/// An error occurring during the parsing of a RIS file.
#[derive(Debug, Clone, Copy)]
pub struct ParseError {
    /// The line number (starting at 1) on which the error occurred.
    pub line_no: usize,
    /// The kind of error
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(line_no: usize, kind: ParseErrorKind) -> Self {
        Self { line_no, kind }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use ParseErrorKind::*;

        match self.kind {
            UnterminatedEntry => write!(f, "Unterminated entry started at line {}", self.line_no),
            InvalidKey => write!(f, "Invalid key at line {}", self.line_no),
            InvalidLine => write!(f, "Invalid line format at line {}", self.line_no),
            DuplicateField => write!(f, "Duplicate fields at line {}", self.line_no),
            InvalidDate => write!(f, "Invalid date format at line {}", self.line_no),
        }
    }
}

impl std::error::Error for ParseError {}

/// The kind of an error occurring during the parsing of a RIS file.
#[derive(Debug, Clone, Copy)]
pub enum ParseErrorKind {
    /// An entry started by `TY` was not terminated by `ER` before another `TY` or EOF was encontered.
    UnterminatedEntry,
    /// An invalid key was encountered.
    InvalidKey,
    /// A line was not in the RIS format of `<letter><letter_or_number><space><space><dash><space><any>*`.
    InvalidLine,
    /// A unique field was present multiple times in a single entry.
    DuplicateField,
    /// A date field was not in the `YYYY/MM/DD/otherinfo` format.
    InvalidDate,
}

struct CurrentEntry {
    entry: Entry,
    start_line_no: usize,
}

fn set_unique_field<T>(field: &mut Option<T>, value: T, line_no: usize) -> Result<(), ParseError> {
    if field.is_some() {
        Err(ParseError::new(line_no, ParseErrorKind::DuplicateField))
    } else {
        *field = Some(value);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn deserialize_one_record() {
        let s = "TY  - JOUR
AU  - Shannon, Claude E.
PY  - 1948/07//
TI  - A Mathematical Theory of Communication
T2  - Bell System Technical Journal
SP  - 379
EP  - 423
VL  - 27
ER  - ";

        let ris = RIS(vec![Entry {
            authors: vec![String::from("Shannon, Claude E.")],
            primary_date: Some(PublicationDate::new(1948, Some(7), None, None)),
            title: Some(String::from("A Mathematical Theory of Communication")),
            secondary_title: Some(String::from("Bell System Technical Journal")),
            start_page: Some(String::from("379")),
            end_page: Some(String::from("423")),
            volume: Some(String::from("27")),
            ..Entry::new(ReferenceType::Journal)
        }]);

        assert_eq!(ris, RIS::from_str(s).unwrap());
    }

    #[test]
    fn deserialize_two_records() {
        let s = "TY  - JOUR
AU  - Shannon, Claude E.
PY  - 1948/07//
TI  - A Mathematical Theory of Communication
T2  - Bell System Technical Journal
SP  - 379
EP  - 423
VL  - 27
ER  - \nTY  - JOUR
T1  - On computable numbers, with an application to the Entscheidungsproblem
A1  - Turing, Alan Mathison
JO  - Proc. of London Mathematical Society
VL  - 47
IS  - 1
SP  - 230
EP  - 265
Y1  - 1937
ER  - ";

        let ris = RIS(vec![
            Entry {
                authors: vec![String::from("Shannon, Claude E.")],
                primary_date: Some(PublicationDate::new(1948, Some(7), None, None)),
                title: Some(String::from("A Mathematical Theory of Communication")),
                secondary_title: Some(String::from("Bell System Technical Journal")),
                start_page: Some(String::from("379")),
                end_page: Some(String::from("423")),
                volume: Some(String::from("27")),
                ..Entry::new(ReferenceType::Journal)
            },
            Entry {
                title: Some(String::from(
                    "On computable numbers, with an application to the Entscheidungsproblem",
                )),
                authors: vec![String::from("Turing, Alan Mathison")],
                journal: Some(String::from("Proc. of London Mathematical Society")),
                volume: Some(String::from("47")),
                issue: Some(String::from("1")),
                start_page: Some(String::from("230")),
                end_page: Some(String::from("265")),
                primary_date: Some(PublicationDate::new(1937, None, None, None)),
                ..Entry::new(ReferenceType::Journal)
            },
        ]);

        assert_eq!(ris, RIS::from_str(s).unwrap());
    }

    #[test]
    fn serialize_one_record() {
        let ris = RIS(vec![Entry {
            authors: vec![String::from("Shannon, Claude E.")],
            primary_date: Some(PublicationDate::new(1948, Some(7), None, None)),
            title: Some(String::from("A Mathematical Theory of Communication")),
            secondary_title: Some(String::from("Bell System Technical Journal")),
            start_page: Some(String::from("379")),
            end_page: Some(String::from("423")),
            volume: Some(String::from("27")),
            ..Entry::new(ReferenceType::Journal)
        }]);

        let s = "TY  - JOUR
T1  - A Mathematical Theory of Communication
T2  - Bell System Technical Journal
A1  - Shannon, Claude E.
Y1  - 1948/07//
SP  - 379
EP  - 423
VL  - 27
ER  - ";

        assert_eq!(ris.to_string(), s);
    }

    #[test]
    fn serialize_two_records() {
        let ris = RIS(vec![
            Entry {
                authors: vec![String::from("Shannon, Claude E.")],
                primary_date: Some(PublicationDate::new(1948, Some(7), None, None)),
                title: Some(String::from("A Mathematical Theory of Communication")),
                secondary_title: Some(String::from("Bell System Technical Journal")),
                start_page: Some(String::from("379")),
                end_page: Some(String::from("423")),
                volume: Some(String::from("27")),
                ..Entry::new(ReferenceType::Journal)
            },
            Entry {
                title: Some(String::from(
                    "On computable numbers, with an application to the Entscheidungsproblem",
                )),
                authors: vec![String::from("Turing, Alan Mathison")],
                journal: Some(String::from("Proc. of London Mathematical Society")),
                volume: Some(String::from("47")),
                issue: Some(String::from("1")),
                start_page: Some(String::from("230")),
                end_page: Some(String::from("265")),
                primary_date: Some(PublicationDate::new(1937, None, None, None)),
                ..Entry::new(ReferenceType::Journal)
            },
        ]);

        let s = "TY  - JOUR
T1  - A Mathematical Theory of Communication
T2  - Bell System Technical Journal
A1  - Shannon, Claude E.
Y1  - 1948/07//
SP  - 379
EP  - 423
VL  - 27
ER  - \nTY  - JOUR
T1  - On computable numbers, with an application to the Entscheidungsproblem
A1  - Turing, Alan Mathison
Y1  - 1937///
SP  - 230
EP  - 265
JF  - Proc. of London Mathematical Society
VL  - 47
IS  - 1
ER  - ";

        assert_eq!(ris.to_string(), s);
    }

}
