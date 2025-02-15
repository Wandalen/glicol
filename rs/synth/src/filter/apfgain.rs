use dasp_graph::{Buffer, Input, Node};
use dasp_ring_buffer as ring_buffer;
use super::super::{GlicolNodeData, NodeData, BoxedNodeSend, mono_node};

type Fixed = ring_buffer::Fixed<Vec<f32>>;

pub struct AllpassGain<const N: usize> {
    delay: f32,
    gain: f32,
    bufx: Fixed,
    bufy: Fixed,
    sr: usize,
}

impl<const N: usize> AllpassGain<N> {
    pub fn new() -> Self {
        Self {
            delay: 0.5,
            gain: 0.5,
            sr: 44100,
            bufx: ring_buffer::Fixed::from(vec![0.0]),
            bufy: ring_buffer::Fixed::from(vec![0.0])
        }
    }

    pub fn delay(self, delay: f32) -> Self {
        let size;
        if delay == 0.0 {
            size = (10.0 * self.sr as f32) as usize;
        } else {
            size = (delay * self.sr as f32) as usize
        };
        Self {
            bufx: ring_buffer::Fixed::from(vec![0.0; size]),
            bufy: ring_buffer::Fixed::from(vec![0.0; size]),
            ..self
        }
    }

    pub fn gain(self, gain: f32) -> Self {
        Self {gain, ..self}
    }

    pub fn sr(self, sr:usize) -> Self {
        Self {sr, ..self}
    }

    pub fn build(self) -> GlicolNodeData<N> {
        mono_node!(N, self)
    }
}

impl<const N: usize> Node<N> for AllpassGain<N> {
    fn process(&mut self, inputs: &[Input<N>], output: &mut [Buffer<N>]) {
        let l = inputs.len();
        if l < 1 { return ()};
        let has_clock = inputs[l-1].buffers()[0][0] as usize % N == 0 && inputs[l-1].buffers()[0][1] == 0.;

        // println!("{}{}",l ,has_clock );
        if l - has_clock as usize > 1 { // has mod
            let insig = inputs[1].buffers()[0].clone();
            let modulator = inputs[0].buffers()[0].clone();
            let new_delay_samples = (modulator[0] / self.sr as f32) as usize;
            let length = self.bufx.len();
            
            for i in 0..N {
                // println!("{:?}", self.buf);
                let xn = insig[i];
                let yn = -self.gain * xn
                + self.bufx[0] + self.gain * self.bufy[0];
                
                // save new input to ring buffer
                self.bufx.push(xn);
                self.bufy.push(yn);
                output[0][i] = yn;
                self.bufx.set_first(length - new_delay_samples);
                self.bufy.set_first(length - new_delay_samples);
            }
        } else {
            for i in 0..N {
                // println!("{:?}", self.buf);
                let xn = inputs[0].buffers()[0][i];
                let yn = -self.gain * xn
                + self.bufx[0] + self.gain * self.bufy[0];
                
                // save new input to ring buffer
                self.bufx.push(xn);
                self.bufy.push(yn);
                output[0][i] = yn;
            }
        }
    }
}