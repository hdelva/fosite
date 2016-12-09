use super::Pointer;

pub struct Collection {
  content: Vec<Chunk>,
}

#[derive(Clone, PartialEq)]
pub struct Chunk {
  minimum: i16,
  maximum: i16,
  representant: Representant,
}

#[derive(Clone, PartialEq)]
pub struct Representant {
	object: Pointer,
	kind: Pointer,
}

impl Chunk {
	fn new(min: i16, max: i16, repr: Representant) -> Chunk {
		Chunk {
			minimum: min,
			maximum: max,
			representant: repr,
		}
	}
}

impl Collection {
  pub fn define(&mut self, definition: Vec<Representant>) {
    for object in definition {
      self.content.push(Chunk::new(1, 1, object))
    }
  }

  pub fn append(&mut self, definition: Representant, min: i16, max: i16) {
  	{
	  	let last = self.content.last_mut();
    
	    match last {
	    	Some(mut element) => {
	    		if element.representant.kind == definition.kind {
			      element.minimum += min;
			      element.maximum += max;
			      return;
			    } 
	    	},
	    	_ => (),
	    }
  	}
  	
  	self.content.push(Chunk::new(min, max, definition))
    
  }

  pub fn prepend(&mut self, definition: Representant, min: i16, max: i16) {
  	{
	    let first = self.content.first_mut();
	    
	    match first {
	    	Some(mut element) => {
	    		if element.representant.kind == definition.kind {
			      element.minimum += min;
			      element.maximum += max;
			    } 
	    	},
	    	_ => (),
	    }
  	}
  	
  	self.content.insert(0, Chunk::new(min, max, definition));
  	
  }

  pub fn get_first_n(&self, n: i16) -> Vec<Vec<&Chunk>> {
    let current = Vec::new();
    return self.fuck(n, current, &self.content, false);
  }

  pub fn get_last_n(&self, n: i16) -> Vec<Vec<&Chunk>> {
    let current = Vec::new();
    let mut result = self.fuck(n, current, &self.content, true);
    result.reverse();
    return result
  }

  //todo
  fn get_exact_n(&self, n: i16) {
    let first = self.get_first_n(n);
    let last = self.get_last_n(n);
    // return intersect
  }

  pub fn slice(&self, start: i16, end: i16) -> Vec<Vec<&Chunk>> {
    let head = self.get_first_n(start);
    let mut start_positions = Vec::new();
    
    for possibility in head {
	    let last_head = possibility.last();
		
	    match last_head {
	      Some(element) => {
	        let mut count = 0;
	        for chunk in possibility.iter().rev() {
	          if element == chunk {
	            count += 1
	          } else {
	            break;
	          }
	        }
	        
	        let mut cloned = possibility.clone();
	        cloned.dedup();
	        
	        if count < element.maximum {
	        	start_positions.push(cloned.len() - 1);
	        } else {
	        	start_positions.push(cloned.len());
	        }
	      },
	      _ => start_positions.push(0),
	    }
    }
        
    let tail = self.get_last_n(end);
    let mut end_positions = Vec::new();
    
    for possibility in tail {
    	let first_tail = possibility.first();
	
	    match first_tail {
	      Some(element) => {
	        let mut count = 0;
	        for chunk in possibility.iter() {
	          if element == chunk {
	            count += 1
	          } else {
	            break;
	          }
	        }
	        
	        let mut cloned = possibility.clone();
	        cloned.dedup();
	        
	        if count < element.maximum {
	        	end_positions.push(self.content.len() - cloned.len() - 1);
	        } else {
	        	end_positions.push(self.content.len() - cloned.len());
	        }
	      },
	      _ => end_positions.push(self.content.len() - 1),
	    }
    }
    
    let mut result = Vec::new();
    
    for start in start_positions {
    	for end in &end_positions {
    		let mut temp = Vec::new();
    		for i in start..*end {
    			temp.push(&self.content[i]);
    		}
    		result.push(temp);
    	}
    }
    
    return result;
  }

  fn fuck<'a, 'b>(&'a self, n: i16, mut current: Vec<&'a Chunk>, chunks: &'a [Chunk], reverse: bool) -> Vec<Vec<&'a Chunk>> {
  	let chunk;
  	
  	if reverse {
	  	chunk = chunks.last();	
  	} else {
  		chunk = chunks.first();
  	}
    
    match chunk {
    	Some(element) => {
    		let mut result = Vec::new();

		    for i in 0..element.minimum {
		      current.push(&element);
		      if current.len() >= n as usize {
		        result.push(current);
		        return result;
		      }
		    }
		
		    for i in 0..element.maximum {
		    	current.push(&element);
		    	
		    	if current.len() >= n as usize{
		    		result.push(current);
		    		return result;
		    	} else {
			    	let mut partial = self.fuck(n, current.clone(), &chunks[1..], reverse);
			    	result.append(&mut partial);	
		    	}
		    	
		    };
		    
		    return result;
    	},
    	_ => panic!("not enough elements to unpack"),
    }
    
    
  }
}