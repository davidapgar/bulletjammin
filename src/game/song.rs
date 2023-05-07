use super::audio::audio_generator::*;
use super::audio::Audio;
use bevy::prelude::*;

pub type Notes = [Option<(i32, RawSource)>; 4];

#[derive(Resource)]
pub struct Song {
    tracks: Vec<Track>,
}

impl Song {
    pub fn note(&self, idx: usize) -> Notes {
        let mut notes: Notes = [None, None, None, None];
        for i in 0..self.tracks.len() {
            if i >= 4 {
                break;
            }

            notes[i] = self.tracks[i].note(idx);
        }
        notes
    }

    pub fn len(&self) -> usize {
        let mut max = 0;
        for track in &self.tracks {
            let len = track.len();
            if len > max {
                max = len;
            }
        }
        max
    }
}

pub struct Track {
    chains: Vec<Chain>,
}

impl Track {
    fn note(&self, mut idx: usize) -> Option<(i32, RawSource)> {
        for chain in &self.chains {
            if idx < chain.len() {
                return chain.note(idx);
            }
            idx -= chain.len();
        }
        None
    }

    fn len(&self) -> usize {
        let mut len = 0usize;

        for chain in &self.chains {
            len += chain.len();
        }

        len
    }
}

pub struct Chain {
    phrases: Vec<Phrase>,
}

impl Chain {
    fn note(&self, mut idx: usize) -> Option<(i32, RawSource)> {
        for phrase in &self.phrases {
            if idx < phrase.len() {
                return phrase.note(idx);
            }
            idx -= phrase.len();
        }
        None
    }

    fn len(&self) -> usize {
        let mut len = 0usize;

        for phrase in &self.phrases {
            len += phrase.len();
        }

        len
    }
}

pub struct Phrase {
    notes: &'static str,
    sound_gen: Box<dyn Fn(f32) -> RawSource + Sync + Send + 'static>,
}

impl Phrase {
    fn new(notes: &'static str) -> Self {
        Self {
            notes,
            sound_gen: Box::new(|frequency| {
                Vca::new(
                    Vco::new(
                        Vcf::new(
                            SquareWave::new(frequency as f32).as_raw(),
                            frequency / 4.,
                            1.0,
                        ),
                        frequency / 2.,
                        Envelope::new(0.3, 0.1, 0.05, 0.1),
                    ),
                    Envelope::new(0.3, 0.05, 0.05, 0.2),
                )
                .as_raw()
            }),
        }
    }

    fn len(&self) -> usize {
        self.notes.len()
    }

    fn note(&self, idx: usize) -> Option<(i32, RawSource)> {
        if idx > self.notes.len() {
            return None;
        }

        if let Some(note) = match self.notes.as_bytes()[idx] as char {
            'c' => Some(0),
            'd' => Some(2),
            'e' => Some(4),
            'f' => Some(5),
            'g' => Some(7),
            'a' => Some(9),
            'b' => Some(11),
            _ => None,
        } {
            let voltage = note as f32 / 120.;
            let frequency = frequency_per_volt(voltage + 0.2);
            Some((note, (self.sound_gen)(frequency)))
        } else {
            None
        }
    }
}

pub fn mary_song() -> Song {
    let phrases = vec![
        Phrase::new("edcdeee_"),
        Phrase::new("ddd_cgg_"),
        Phrase::new("edcdeee_"),
        Phrase::new("ddedc___"),
    ];
    let chain = Chain { phrases };
    let track = Track {
        chains: vec![chain],
    };
    let song = Song {
        tracks: vec![track],
    };
    song
}
