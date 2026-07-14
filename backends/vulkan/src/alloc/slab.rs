const END_OF_LIST: usize = usize::MAX;

pub struct Slab<T> {
    entries: Vec<Entry<T>>,
    free_head: usize,
    len: usize,
}

enum Entry<T> {
    Occupied(T),
    Vacant(usize),
}

impl<T> Slab<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_head: END_OF_LIST,
            len: 0,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            entries: Vec::with_capacity(cap),
            free_head: END_OF_LIST,
            len: 0,
        }
    }

    pub fn insert_at(&mut self, index: usize, value: T) -> Option<T> {
        if index >= self.entries.len() {
            self.entries.resize_with(index + 1, || Entry::Vacant(END_OF_LIST));
        }

        let old = std::mem::replace(&mut self.entries[index], Entry::Occupied(value));

        match old {
            Entry::Occupied(old_value) => Some(old_value),
            Entry::Vacant(_) => {
                self.len += 1;
                None
            }
        }
    }

    pub fn insert(&mut self, value: T) -> usize {
        self.len += 1;

        if self.free_head == END_OF_LIST {
            let index = self.entries.len();
            self.entries.push(Entry::Occupied(value));
            index
        }else {
            let index = self.free_head;
            let next_free = match &self.entries[index] {
                Entry::Vacant(next_free) => *next_free,
                Entry::Occupied(_) => unreachable!(),
            };

            self.free_head = next_free;
            self.entries[index] = Entry::Occupied(value);
            index
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        match self.entries.get(index)? {
            Entry::Vacant(_) => return None,
            Entry::Occupied(_) => {}
        }

        let old = std::mem::replace(&mut self.entries[index], Entry::Vacant(self.free_head));

        self.free_head = index;
        self.len -= 1;

        match old {
            Entry::Occupied(value) => Some(value),
            Entry::Vacant(_) => unreachable!(),
        }
    }

    pub fn take(&mut self, index: usize) -> Option<T> {
        match self.entries.get(index)? {
            Entry::Vacant(_) => return None,
            Entry::Occupied(_) => {}
        }

        let old = std::mem::replace(&mut self.entries[index], Entry::Vacant(self.free_head));

        self.free_head = index;
        self.len -= 1;

        match old {
            Entry::Occupied(value) => Some(value),
            Entry::Vacant(_) => unreachable!(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        match self.entries.get(index)? {
            Entry::Occupied(value) => Some(value),
            Entry::Vacant(_) => None,
        }
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match self.entries.get_mut(index)? {
            Entry::Occupied(value) => Some(value),
            Entry::Vacant(_) => None,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
