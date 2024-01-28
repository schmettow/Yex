pub use std::time::{Instant, Duration};
pub use futures_timer::Delay;
pub use std::sync::{Arc,Mutex};
pub use isolang::Language;
pub use futures;


/// Input events
pub type Text = String;
pub type Key = char;

pub enum NaviEvent{Back, Forward, Quit}

pub enum Event {
    Response(),
    InputEvent,
    AdvanceAfter(Duration)
}

/// Demo program
/// 
/// cycles through a brief demo experiment
use session::*;
pub fn demo(session: Arc<Mutex<Session>>){
    let mut session = session.lock().unwrap();
    session.state = State::Welcome;
    println!("Welcome");
    Delay::new(Duration::from_millis(500));
    for block in &mut session.exp.blocks{
        println!("Block");
        block.run();
    }
    session.state = State::Goodbye;
}


/// Building sessions
/// 
/// A session is the whole encounter of a participant with an experiment.
/// 
/// + composed of a Participant and Experiment object.
/// + runs linearly through the steps of the experiment
/// + sending high-level events

 
pub mod session {
    use super::{Instant, Language, Text};
    use super::block::Block;

    pub struct Session {
        pub id: Instant,
        pub part: Participant,
        pub exp: Experiment,
        pub state: State,
    }

    pub enum State {
        Init,
        Welcome,
        Consent,
        Demographics,
        Blocks(Block),
        Goodbye
    }

    impl Session {
        pub fn new(exp: Experiment, part: Participant) -> Self{
            Session{id: Instant::now(),
                    part: part,
                    exp: exp,
                    state: State::Init}
        }
    }


    #[derive(Clone)]
    pub struct Participant {
        pub id: usize,
        pub age: i8,
        pub gender: Gender,
        pub language: Language,
    }

    impl Default for Participant {
        fn default() -> Self {
            Self { id: 0, age: 42, gender: Gender::Straight(Sex::Male), language: Language::default() }
        }
    }

    #[derive(Clone)]
    pub enum Sex {
        Male,
        Female,
    }

    #[derive(Clone)]
    pub enum Gender {
        Straight(Sex),
        Gay(Sex),
        Bi(Sex),
        Asexual(Sex)
    }


    /// Experiments are composed of blocks
    /// 
    /// An Experiment is a container for trials arranged in blocks.
    /// 
    /// data-only class as Session is doing the run()


    #[derive(Clone)]
    pub struct Experiment {
        pub id: String,
        pub blocks: Vec<Block>,
        pub instructions: Text,
        pub random: bool,
    }

    impl Default for Experiment {
        fn default() -> Self {
            Self {  id: "Stroop".into(), 
                    blocks: vec![Block::default();2],
                    instructions: "Say the color of the word!".into(),
                    random: false,}
        }
}



}


/// Block level

pub mod block { 
    use super::trial::{Trial, Observation};
    use super::{Duration, Instant, Delay, Key, Text};

    /// A Block is a sequences of Trials
    /// 
    /// with a prelude and relax frame.
    /// 
    /// + running through trials
    /// + sending block-level events
    /// 
    #[derive(Clone)]
    pub struct Block{
        pub id: Instant,
        pub trials: Vec<Trial>,
        pub random: bool,
        pub prelude: Prelude,
        pub relax: Relax,
        pub state: State,
    }

    
    impl Default for Block {
        fn default() -> Self {
            let trials = vec![Trial::default(); 3];
            Block{  id: Instant::now(),
                    trials: trials, 
                    random: false, 
                    prelude: Prelude::Blank(Duration::from_millis(1000)),
                    relax: Relax::Wait(Duration::from_millis(2000)),
                    state: State::Init,
                }
        }
    }

    #[derive(Clone, PartialEq)]    
    /// Block states
    /// 
    pub enum State {
        Init,
        Prelude,
        Present(usize), // trial number
        Relax
    }

    /// Preludes types for Blocks
    /// 
    #[derive(Clone, PartialEq, Debug)]
    pub enum Prelude {
        Now,
        Blank(Duration),
        Instruct(Duration, Text),
        InstructKeys(Vec<Key>, Text)
    }

    /// Relax types for Blocks
    ///
    #[derive(Clone)]
    pub enum Relax {
        Now,
        Wait(Duration),
        Keys(Vec<Key>),
        KeysMaxWait(Vec<Key>, Duration)
    }

    
    impl Block {
    /// Run a block
    /// 
    /// runs through one block and its trials
    /// returns a vector of Observations (Trial + Response)
    /// 1. initialize the output vector
    /// 2. do the prelude
    /// 3. cycle through trials and 
    /// 4. Run the relax period
    /// 
        pub fn run(&mut self) -> Vec<Observation> {
            let mut out: Vec<Observation> = Vec::new();
            self.state = State::Prelude;            
            match self.prelude.clone() {
                Prelude::Now
                    => {},
                Prelude::Instruct(dur, _) 
                    => {Delay::new(dur);},
                _   => todo!(),
            }

            for trial in self.trials.clone(){
                let obs = trial.clone().run();
                out.push(obs);
            }

            self.state = State::Relax;
            match self.relax {
                Relax::Now => {},
                Relax::Wait(dur) 
                    => {Delay::new(dur);},
                _   => {todo!();}
            }
            out
        }
    }


}


/// Trial-level
/// 

pub mod trial { 
    use super::{Duration, Delay, Key};

    /// A trial is a Stimulus with a Prelude and Advance frame
    /// 

    #[derive(Clone, PartialEq)]
    pub struct Trial {
        pub prelude: Prelude,
        pub stimulus: Stimulus,
        pub advance: Advance,
        pub state: State
    }
    
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum State {
        Init,
        Prelude,
        Present,
        Feedback
    }
    
    impl Default for Trial {
        fn default() -> Self {
            Self {  state: State::Init,
                    prelude: Prelude::Blank(Duration::from_micros(500)) ,
                    stimulus: Stimulus::Blank(Duration::from_micros(500)),
                    advance: Advance::Wait(Duration::from_millis(500))}
        }
    }
    
    impl Trial {
        pub fn prepare(&mut self) -> Self{
            self.stimulus.load();
            self.clone()
        }
        pub fn run(&mut self) -> Observation {
            self.prepare();
            self.state = State::Prelude;
            match self.prelude {
                Prelude::Now => {},
                Prelude::Blank(dur) | Prelude::Fix(dur) 
                    => {Delay::new(dur);},
                Prelude::Prime(_,_) => todo!(),
            }
            self.state = State::Present;
            // Emulating the incoming response from the participant.
            // 
            // Here we will have time-outs and user events intermixed.
            // Would be nice to have some async here, maybe 
            // block_on(select())
            Delay::new(Duration::from_millis(500));
            let response = Response::Choice('y');
            Observation::new(self.clone(), response)
        }
    }

    #[derive(Clone, PartialEq)]
    pub struct Observation {
        pub trial: Trial,
        pub response: Response,
    }
    
    /// An observation is composed of a trial and an observation

    // We will need access to higher level information
    // to add part and exp level data

    impl Observation {
        pub fn new(trial: Trial, response: Response) -> Self {
            Self{trial: trial, response: response}
        }
    }

    use image;
    #[derive(Clone, PartialEq)]
    pub enum Stimulus {
        Blank(Duration),
        Text(Duration, i8, [i8; 3]),
        Image(Duration, image::RgbaImage, [usize; 4]),
    }

    impl Stimulus{
        pub fn load(&mut self) -> &Self
        {self}
    }

    #[derive(Clone, PartialEq)]
    pub enum Prelude {
        Now,
        Blank(Duration),
        Fix(Duration),
        Prime(Duration, Stimulus),
    }

    #[derive(Clone, PartialEq)]
    pub enum Advance {
        Wait(Duration),
        Keys(Vec<Key>),
        KeysMaxWait(Vec<Key>, Duration)
    }

    #[derive(Clone, Copy, PartialEq)]
    pub enum Response {
        RT(Duration),
        RTCorrect(Duration, bool),
        Choice(Key),
        Graded(f32),
        TooLate,
    }

    #[derive(Clone, Copy, PartialEq)]
    pub enum Feedback{Correct, Incorrect, ThankYou}
}



/// Output
/// 
/// in terms of
/// + event stream
/// + observations

pub mod output {
    use super::{Key, Duration};
    use super::session::Participant;
    use super::trial::{Stimulus, Response};

    #[allow(dead_code)]
    enum YexError {
        FileNotFound(Stimulus),
        PartInterrupt(Participant),
    }

    #[allow(dead_code)]
    enum YldEvent {
        Error(YexError),
        Block(usize),
        Relax(Duration),
        FixCross(Duration),
        StimPresented(Stimulus),
        KeyPress(Key),
        Response(Response),
    }


     
    /*struct YldRecord {
        time: Instant,
        event: YldEvent
    }


    impl YldEvent {
        fn to_csv(self) ->  String {
            format!("{},{}", "time", "event")
        }
    }*/

}