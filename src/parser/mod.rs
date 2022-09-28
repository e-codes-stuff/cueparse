use std::path::PathBuf;

use pest_consume::{match_nodes, Error, Parser};

use crate::{Cue, FileFormat, Frames, Track, TrackFlags, TrackIndex, TrackMode};

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

struct CueFile {
    path: PathBuf,
    format: FileFormat,
}

impl CueFile {
    pub fn new(path: impl Into<PathBuf>, format: FileFormat) -> Self {
        Self {
            path: path.into(),
            format,
        }
    }
}

enum GlobalProperty {
    Catalog(String),
    CdTextFile(PathBuf),
    File(CueFile),
    Performer(String),
    Songwriter(String),
    Title(String),
    Rem(String),
    Arranger(String),
}

enum TrackProperty {
    File(CueFile),
    Flags(TrackFlags),
    Performer(String),
    SongWriter(String),
    Title(String),
    Index(TrackIndex),
    Isrc(String),
    PreGap(Frames),
    PostGap(Frames),
    Rem(String),
    Arranger(String),
}

#[derive(Parser)]
#[grammar = "./parser/cue.pest"]
struct CueParser;

#[pest_consume::parser]
impl CueParser {
    fn EOI(_i: Node) -> Result<()> {
        Ok(())
    }

    fn string(i: Node) -> Result<String> {
        Ok(i.as_str().trim_matches('"').to_string())
    }

    fn integer(i: Node) -> Result<usize> {
        Ok(i.as_str().parse().map_err(|e| i.error(e))?)
    }

    fn msf_time(i: Node) -> Result<Frames> {
        match_nodes!(i.into_children();
            [integer(m), integer(s), integer(f)] => Ok(Frames::from_msf(m, s, f))
        )
    }

    fn time(i: Node) -> Result<Frames> {
        match_nodes!(i.into_children();
            [msf_time(time)] => Ok(time),
            [integer(frames)] => Ok(Frames::new(frames)),
        )
    }

    fn catalog_number(i: Node) -> Result<String> {
        Ok(i.as_str().to_string())
    }

    fn file_format(i: Node) -> Result<FileFormat> {
        let file_format = match i.as_str() {
            "BINARY" => FileFormat::Binary,
            "MOTOROLA" => FileFormat::Motorola,
            "AIFF" => FileFormat::Aiff,
            "WAVE" => FileFormat::Wave,
            "MP3" => FileFormat::Mp3,
            _ => FileFormat::Unspecified,
        };

        Ok(file_format)
    }

    fn flag(i: Node) -> Result<TrackFlags> {
        let flag = match i.as_str() {
            "DCP" => TrackFlags::DIGITAL_COPY_PERMITTED,
            "4CH" => TrackFlags::FOUR_CHANNEL,
            "PRE" => TrackFlags::PRE_EMPHASIS_ENABLED,
            "SCMS" => TrackFlags::SERIAL_COPY_MANAGEMENT_SYSTEM,
            _ => return Err(i.error("Expected track flag")),
        };

        Ok(flag)
    }

    fn track_mode(i: Node) -> Result<TrackMode> {
        use TrackMode::*;

        let mode = match i.as_str() {
            "AUDIO" => Audio,
            "CDG" => Cdg,
            "MODE1/2048" => Mode1_2048,
            "MODE1/2352" => Mode1_2352,
            "MODE2/2336" => Mode2_2336,
            "MODE2/2352" => Mode2_2352,
            "CDI/2336" => Cdi_2336,
            "CDI/2352" => Cdi_2352,
            _ => return Err(i.error("Expected track mode")),
        };

        Ok(mode)
    }

    fn isrc_code(i: Node) -> Result<String> {
        Ok(i.as_str().to_string())
    }

    fn file(i: Node) -> Result<CueFile> {
        match_nodes!(i.into_children();
            [string(path), file_format(format)] => Ok(CueFile::new(path, format)),
            [string(path)] => Ok(CueFile::new(path, FileFormat::Unspecified))
        )
    }

    fn catalog(i: Node) -> Result<String> {
        match_nodes!(i.into_children();
            [catalog_number(number)] => Ok(number)
        )
    }

    fn cd_text_file(i: Node) -> Result<PathBuf> {
        match_nodes!(i.into_children();
            [string(path)] => Ok(PathBuf::from(path))
        )
    }

    fn flags(i: Node) -> Result<TrackFlags> {
        let mut result_flags = TrackFlags::empty();

        match_nodes!(i.into_children();
            [flag(flags)..] => {
                result_flags.extend(flags);
                Ok(result_flags)
            }
        )
    }

    fn performer(i: Node) -> Result<String> {
        match_nodes!(i.into_children();
            [string(val)] => Ok(val)
        )
    }

    fn songwriter(i: Node) -> Result<String> {
        match_nodes!(i.into_children();
            [string(val)] => Ok(val)
        )
    }

    fn title(i: Node) -> Result<String> {
        match_nodes!(i.into_children();
            [string(val)] => Ok(val)
        )
    }

    fn index(i: Node) -> Result<TrackIndex> {
        match_nodes!(i.into_children();
            [integer(index), time(time)] => Ok(TrackIndex {
                index,
                time: Some(time),
            }),

            [integer(index)] => Ok(TrackIndex {
                index,
                time: None,
            }),
        )
    }

    fn pregap(i: Node) -> Result<Frames> {
        match_nodes!(i.into_children();
            [time(gap)] => Ok(gap)
        )
    }

    fn postgap(i: Node) -> Result<Frames> {
        match_nodes!(i.into_children();
            [time(gap)] => Ok(gap)
        )
    }

    fn isrc(i: Node) -> Result<String> {
        match_nodes!(i.into_children();
            [isrc_code(code)] => Ok(code)
        )
    }

    fn rem(i: Node) -> Result<String> {
        match_nodes!(i.into_children();
            [rem_text(comment)] => Ok(comment)
        )
    }

    fn rem_text(i: Node) -> Result<String> {
        Ok(i.as_str().into())
    }

    // CDTEXT commands
    fn arranger(i: Node) -> Result<String> {
        match_nodes!(i.into_children();
            [string(arranger)] => Ok(arranger)
        )
    }

    // global section
    fn global_section(i: Node) -> Result<Cue> {
        use GlobalProperty::*;

        match_nodes!(i.into_children();
            [global_property(properties)..] => {
                let mut cue = Cue::default();

                properties.for_each(|property| {
                    match property {
                        Catalog(catalog) => cue.catalog = Some(catalog),
                        CdTextFile(path) => cue.cd_text_file = Some(path),
                        File(file) => {
                            cue.path = Some(file.path);
                            cue.format = file.format;
                        }
                        Performer(performer) => cue.performer = Some(performer),
                        Songwriter(songwriter) => cue.songwriter = Some(songwriter),
                        Title(title) => cue.title = Some(title),
                        Rem(comment) => cue.comments.push(comment),
                        Arranger(arranger) => cue.arranger = Some(arranger),
                    }
                });

                Ok(cue)
            }
        )
    }

    fn global_property(i: Node) -> Result<GlobalProperty> {
        use GlobalProperty::*;

        let property = match_nodes!(i.into_children();
            [catalog(catalog)] => Catalog(catalog),
            [cd_text_file(cdtext)] => CdTextFile(cdtext),
            [file(file)] => File(file),
            [performer(performer)] => Performer(performer),
            [songwriter(writer)] => Songwriter(writer),
            [title(title)] => Title(title),
            [rem(comment)] => Rem(comment),
            [arranger(arranger)] => Arranger(arranger),
        );

        Ok(property)
    }

    // track section
    fn track_list(i: Node) -> Result<Vec<Track>> {
        match_nodes!(i.into_children();
            [track(tracks)..] => Ok(tracks.collect())
        )
    }

    fn track(i: Node) -> Result<Track> {
        use TrackProperty::*;

        match_nodes!(i.into_children();
            [track_command(mut track), track_property(properties).., _] => {
                properties.for_each(|property|
                    match property {
                        File(file) => track.set_file(file.path, file.format),
                        Flags(flags) => track.flags |= flags,
                        Performer(performer) => track.performer = Some(performer),
                        SongWriter(songwriter) => track.songwriter = Some(songwriter),
                        Title(title) => track.title = Some(title),
                        Index(index) => track.indices.push(index),
                        Isrc(isrc) => track.isrc = Some(isrc),
                        PreGap(pregap) => track.pregap = Some(pregap),
                        PostGap(postgap) => track.postgap = Some(postgap),
                        Rem(comment) => track.comments.push(comment),
                        Arranger(arranger) => track.arranger = Some(arranger)
                    }
                );

                Ok(track)
            }
        )
    }

    fn track_command(i: Node) -> Result<Track> {
        match_nodes!(i.into_children();
            [integer(track_index), track_mode(mode)] => Ok(Track::new(track_index as u8, mode)),
        )
    }

    fn track_property(i: Node) -> Result<TrackProperty> {
        let property = match_nodes!(i.into_children();
            [file(track_file)] => TrackProperty::File(track_file),
            [flags(flags)] => TrackProperty::Flags(flags),
            [performer(performer)] => TrackProperty::Performer(performer),
            [songwriter(songwriter)] => TrackProperty::SongWriter(songwriter),
            [title(title)] => TrackProperty::Title(title),
            [index(index)] => TrackProperty::Index(index),
            [isrc(isrc)] => TrackProperty::Isrc(isrc),
            [pregap(pregap)] => TrackProperty::PreGap(pregap),
            [postgap(postgap)] => TrackProperty::PostGap(postgap),
            [rem(rem)] => TrackProperty::Rem(rem),
            [arranger(arranger)] => TrackProperty::Arranger(arranger),
        );

        Ok(property)
    }

    // entry point
    fn cue(i: Node) -> Result<Cue> {
        match_nodes!(i.into_children();
            [global_section(mut cue), track_list(tracks), EOI(_)] => {
                cue.tracks = tracks;
                Ok(cue)
            }
        )
    }
}

pub(crate) fn parse_cue(i: impl AsRef<str>) -> std::result::Result<Cue, crate::Error> {
    let nodes = CueParser::parse(Rule::cue, i.as_ref())?;


    Ok(CueParser::cue(nodes.single()?)?)
}
