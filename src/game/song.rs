use super::audio::audio_generator::*;
use super::audio::Audio;
use bevy::prelude::*;

pub type Notes = [Option<(i32, RawSource)>; 4];

// TODO: allow offset to eigth/quarter?

#[derive(Resource)]
pub struct Song {
    tracks: Vec<Track>,
}

impl Song {
    pub fn note(&self, idx: usize, chain: usize) -> Notes {
        let mut notes: Notes = [None, None, None, None];
        for i in 0..self.tracks.len() {
            if i >= 4 {
                break;
            }

            notes[i] = self.tracks[i].note(idx, chain);
        }
        notes
    }

    pub fn len(&self, chain: usize) -> usize {
        let mut max = 0;
        for track in &self.tracks {
            let len = track.len(chain);
            if len > max {
                max = len;
            }
        }
        max
    }

    pub fn max_chains(&self) -> usize {
        let mut max = 0;
        for track in &self.tracks {
            let len = track.chains.len();
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
    fn note(&self, idx: usize, chain: usize) -> Option<(i32, RawSource)> {
        if chain >= self.chains.len() {
            None
        } else {
            self.chains[chain].note(idx)
        }
    }

    fn len(&self, chain: usize) -> usize {
        if chain >= self.chains.len() {
            0
        } else {
            self.chains[chain].len()
        }
    }
}

pub struct Chain {
    phrases: Vec<Phrase>,
}

impl Chain {
    fn note(&self, mut idx: usize) -> Option<(i32, RawSource)> {
        if idx > self.len() {
            idx = idx % self.len();
        }

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

pub enum PhraseType {
    Quarter,
    Eigth,
    Sixteenth,
}

impl PhraseType {
    fn mult(&self) -> usize {
        match self {
            PhraseType::Quarter => 4,
            PhraseType::Eigth => 2,
            PhraseType::Sixteenth => 1,
        }
    }

    /// `true` if this index is represented in this phrase type
    /// eg: In `Quarter`, only idx % 4 == 0 on "in_phrase"
    fn in_phrase(&self, idx: usize) -> bool {
        idx % self.mult() == 0
    }
}

pub struct Phrase {
    notes: &'static str,
    phrase_type: PhraseType,
    sound_gen: Box<dyn Fn(f32) -> RawSource + Sync + Send + 'static>,
}

impl Phrase {
    fn new<G>(notes: &'static str, phrase_type: PhraseType, sound_gen: G) -> Self
    where
        G: Fn(f32) -> RawSource + Sync + Send + 'static,
    {
        Self {
            notes,
            phrase_type,
            sound_gen: Box::new(sound_gen),
        }
    }

    fn quarter<G>(notes: &'static str, sound_gen: G) -> Self
    where
        G: Fn(f32) -> RawSource + Sync + Send + 'static,
    {
        Self::new(notes, PhraseType::Quarter, sound_gen)
    }

    fn eigth<G>(notes: &'static str, sound_gen: G) -> Self
    where
        G: Fn(f32) -> RawSource + Sync + Send + 'static,
    {
        Self::new(notes, PhraseType::Eigth, sound_gen)
    }

    fn sixteenth<G>(notes: &'static str, sound_gen: G) -> Self
    where
        G: Fn(f32) -> RawSource + Sync + Send + 'static,
    {
        Self::new(notes, PhraseType::Sixteenth, sound_gen)
    }

    fn len(&self) -> usize {
        self.notes.len() * self.phrase_type.mult()
    }

    fn note(&self, mut idx: usize) -> Option<(i32, RawSource)> {
        if !self.phrase_type.in_phrase(idx) {
            return None;
        }

        idx = idx / self.phrase_type.mult();
        if idx > self.notes.len() {
            return None;
        }

        let note_byte = self.notes.as_bytes()[idx];
        if let Some(note) = match note_byte as char {
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
        } else if note_byte >= '0' as u8 && note_byte <= '9' as u8 {
            let byte = note_byte - '0' as u8;
            let voltage = byte as f32 / 120.;
            let frequency = frequency_per_volt(voltage);
            Some((byte as i32, (self.sound_gen)(frequency)))
        } else {
            None
        }
    }
}

fn square_horn(frequency: f32) -> RawSource {
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
}

fn kick(frequency: f32) -> RawSource {
    let kick_env = Envelope::new(1.0, 0.001, 0.1, 0.3);
    let freq_env = Envelope::new(0.02, 0.0, 0.0, 0.2);
    let vco = Vco::new(TriangleWave::new(frequency), frequency, freq_env);
    let osc = Attenuator::new(vco, 2.0);
    let vca = Vca::new(osc, kick_env);

    vca.as_raw()
}

fn snare(_frequency: f32) -> RawSource {
    let snare_env = Envelope::new(0.2, 0.001, 0.0, 0.3);
    let osc = NoiseLFSR::new(20000.);
    let vca = Vca::new(osc, snare_env);
    vca.as_raw()
}

// TODO: Convert all of these to voltage for easier drum type selection
fn drum(frequency: f32) -> RawSource {
    if frequency < 100. {
        kick(frequency)
    } else {
        snare(frequency)
    }
}

pub fn mary_song() -> Song {
    Song {
        tracks: vec![
            // melody
            Track {
                chains: vec![Chain {
                    phrases: vec![
                        Phrase::eigth("edcdeee_", square_horn),
                        Phrase::eigth("ddd_cgg_", square_horn),
                        Phrase::eigth("edcdeee_", square_horn),
                        Phrase::eigth("ddedc___", square_horn),
                    ],
                }],
            },
            // drums
            Track {
                chains: vec![
                    Chain {
                        phrases: vec![Phrase::sixteenth("009_90_0_0_0009_", drum)],
                    },
                    Chain {
                        phrases: vec![Phrase::quarter("1234", drum), Phrase::quarter("4321", drum)],
                    },
                ],
            },
        ],
    }
}
