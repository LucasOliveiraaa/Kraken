use std::sync::Arc;

use glow::HasContext;

use crate::Gpu;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum QueryTarget {
    TimeElapsed,
    AnySamplesPassed,
    AnySamplesPassedConservative,
    PrimitivesGenerated,
    TransformFeedbackPrimitivesWritten,
    SamplesPassed,
}

impl Into<u32> for QueryTarget {
    fn into(self) -> u32 {
        match self {
            QueryTarget::TimeElapsed => glow::TIME_ELAPSED,
            QueryTarget::AnySamplesPassed => glow::ANY_SAMPLES_PASSED,
            QueryTarget::AnySamplesPassedConservative => glow::ANY_SAMPLES_PASSED_CONSERVATIVE,
            QueryTarget::PrimitivesGenerated => glow::PRIMITIVES_GENERATED,
            QueryTarget::TransformFeedbackPrimitivesWritten => {
                glow::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN
            }
            QueryTarget::SamplesPassed => glow::SAMPLES_PASSED,
        }
    }
}

pub struct Query {
    gpu: Arc<Gpu>,
    query: glow::Query,
    target: QueryTarget,
}

impl Query {
    pub fn new(gpu: Arc<Gpu>, target: QueryTarget) -> Result<Self, String> {
        unsafe {
            let gl = gpu.context();
            let query = gl.create_query().map_err(|e| e.to_string())?;
            Ok(Self { gpu, query, target })
        }
    }

    pub fn target(&self) -> QueryTarget {
        self.target
    }

    pub fn begin(&self) {
        unsafe {
            let gl = self.gpu.context();
            gl.begin_query(self.target.into(), self.query);
        }
    }

    pub fn end(&self) {
        unsafe {
            let gl = self.gpu.context();
            gl.end_query(self.target.into());
        }
    }

    pub fn is_result_available(&self) -> bool {
        unsafe {
            let gl = self.gpu.context();
            gl.get_query_parameter_u32(self.query, glow::QUERY_RESULT_AVAILABLE) != 0
        }
    }

    pub fn get_u32(&self) -> Option<u32> {
        unsafe {
            let gl = self.gpu.context();
            if self.is_result_available() {
                Some(gl.get_query_parameter_u32(self.query, glow::QUERY_RESULT))
            } else {
                None
            }
        }
    }

    pub fn get_u64(&self) -> Option<u64> {
        unsafe {
            let gl = self.gpu.context();
            if self.is_result_available() {
                Some(gl.get_query_parameter_u64(self.query, glow::QUERY_RESULT))
            } else {
                None
            }
        }
    }

    pub fn delete(&self) {
        unsafe {
            let gl = self.gpu.context();
            gl.delete_query(self.query);
        }
    }
}
