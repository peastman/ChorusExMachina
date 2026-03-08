// Copyright 2026 by Peter Eastman
//
// This file is part of Chorus Ex Machina.
//
// Chorus Ex Machina is free software: you can redistribute it and/or modify it under the terms
// of the GNU Lesser General Public License as published by the Free Software Foundation, either
// version 2.1 of the License, or (at your option) any later version.
//
// Chorus Ex Machina is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See
// the GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License along with Chorus Ex Machina.
// If not, see <https://www.gnu.org/licenses/>.

use realfft::{RealFftPlanner, RealToComplex, ComplexToReal};
use rustfft::num_complex::Complex;
use std::sync::Arc;

/// This is a convolutional reverb.
pub struct Reverb {
    input_ring: Vec<f32>,
    output_ring: Vec<f32>,
    position: usize,
    ir: Vec<f32>,
    real_temp: Vec<f32>,
    complex_temp: Vec<Complex<f32>>,
    scratch: Vec<Complex<f32>>,
    blocks: Vec<Block>
}

impl Reverb {
    /// Create a reverb to convolve an input signal with an IR in real time. 
    pub fn new(ir: &Vec<f32>, fft_planner: &mut RealFftPlanner::<f32>) -> Self {
        // Extend the IR length to the next power of 2.

        let mut width = 1;
        while width < ir.len() {
            width *= 2;
        }
        let mut ir = ir.clone();
        ir.resize(width, 0.0);

        // Prepare for FFTs.

        let fft_inverse = fft_planner.plan_fft_inverse(2*width);
        let mut input = vec![0.0; 2*width];
        let mut scratch = vec![];

        // Build a series of blocks, each twice as large as the previous one, to convolve
        // parts of the IR with the input.

        let direct_width = usize::min(width, 16);
        let mut blocks = vec![];
        let mut block_width = 2*direct_width;
        while block_width <= width {
            // Create a block to convolve with the second half of (the remaining part of) the IR.

            let fft = fft_planner.plan_fft_forward(block_width);
            if scratch.len() < fft.get_scratch_len() {
                scratch.resize(fft.get_scratch_len(), Complex::<f32>::new(0.0, 0.0));
            }
            input[..block_width/2].copy_from_slice(&ir[block_width/2..block_width]);
            for i in block_width/2..block_width {
                input[i] = 0.0;
            }
            let mut output = fft.make_output_vec();
            match fft.process_with_scratch(&mut input[..block_width], &mut output, &mut scratch) {
                Ok(_) => {}
                Err(message) => {println!["{}", message]}
            }
            blocks.push(Block::new(output, block_width, fft_planner));
            block_width *= 2;
        }
        ir.truncate(direct_width);

        Self {
            input_ring: vec![0.0; 2*width],
            output_ring: vec![0.0; 2*width],
            position: 0,
            ir: ir,
            real_temp: input,
            complex_temp: fft_inverse.make_input_vec(),
            scratch: scratch,
            blocks: blocks
        }
    }

    /// Compute the convolution.  This function takes the next input samples and returns
    /// the next output sample.
    pub fn process(&mut self, input: f32) -> f32 {
        // For the initial part of the IR, it's faster to convolve directly instead
        // of using FFTs.

        let mask = self.output_ring.len()-1;
        self.input_ring[self.position] = input;
        for i in 0..self.ir.len() {
            self.output_ring[(self.position+i)&mask] += input*self.ir[i];
        }

        // Process the blocks.

        for block in self.blocks.iter() {
            let half_width = block.width/2;
            if self.position % half_width == half_width-1 {
                // Copy the input to a buffer and transform it.

                self.real_temp[..half_width].copy_from_slice(&self.input_ring[self.position+1-half_width..self.position+1]);
                for i in half_width..block.width {
                    self.real_temp[i] = 0.0;
                }
                let spectrum_width = half_width + 1;
                match block.fft_forward.process_with_scratch(&mut self.real_temp[..block.width], &mut self.complex_temp[..spectrum_width], &mut self.scratch) {
                    Ok(_) => {}
                    Err(message) => {println!["{}", message]}
                }

                // Perform the convolution in frequency space by multiplying the transformed input and IR.

                for i in 0..block.ir.len() {
                    self.complex_temp[i] *= block.ir[i];
                }

                // Transform back and add it to the output buffer.

                match block.fft_inverse.process_with_scratch(&mut self.complex_temp[..spectrum_width], &mut self.real_temp[..block.width], &mut self.scratch) {
                    Ok(_) => {}
                    Err(message) => {println!["{}", message]}
                }
                let scale = 1.0/block.width as f32;
                for i in 0..block.width {
                    self.output_ring[(self.position+1+i)&mask] += scale*self.real_temp[i];
                }
            }
        }

        // Return the result.

        let result = self.output_ring[self.position];
        self.output_ring[self.position] = 0.0;
        self.position = (self.position+1)&mask;
        result
    }
}

/// This contains data for convolving the input with a block of the IR.
struct Block {
    ir: Vec<Complex<f32>>,
    width: usize,
    fft_forward: Arc<dyn RealToComplex<f32>>,
    fft_inverse: Arc<dyn ComplexToReal<f32>>
}

impl Block {
    fn new(ir: Vec<Complex<f32>>, width: usize, fft_planner: &mut RealFftPlanner::<f32>) -> Self {
        Self {
            ir: ir,
            width: width,
            fft_forward: fft_planner.plan_fft_forward(width),
            fft_inverse: fft_planner.plan_fft_inverse(width)
        }
    }
}