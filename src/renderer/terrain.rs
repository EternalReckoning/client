pub struct HeightMap {
    pub size: usize,
    scale: f32,
    data: Vec<f32>,
}

impl HeightMap {
    pub fn new(data: Vec<f32>, size: usize, scale: f32) -> HeightMap {
        assert_eq!(data.len(), size*size);
        HeightMap { size, scale, data }
    }

    pub fn len(&self) -> usize {
        self.size * self.size
    }

    pub fn get(&self, x: usize, y: usize) -> Option<f32> {
        if x >= self.size || y >= self.size {
            return None;
        }

        Some(*self.data.get(x + y * self.size).unwrap() * self.scale)
    }

    pub fn vertices(&self) -> Vec<rendy::mesh::Position> {
        let mut res = Vec::with_capacity(self.data.len());

        for y in 0..self.size {
            for x in 0..self.size {
                let value = -(*self.data.get(y*self.size + x).unwrap());
                res.push([x as f32 * 1.0, value * self.scale, y as f32 * 1.0].into());
            }
        }

        res
    }

    pub fn colors(&self) -> Vec<rendy::mesh::Color> {
        let mut res = Vec::with_capacity(self.data.len());

        for y in 0..self.size {
            for x in 0..self.size {
                let value = *self.data.get(y*self.size + x).unwrap();
                res.push([value, value, value, 1.0].into());
            }
        }

        res
    }

    pub fn indices(&self) -> Vec<u32> {
        let mut indices = Vec::with_capacity(
            (self.size - 1) * (self.size - 1) * 6
        );

        for y in 0..(self.size - 1) {
            let y_offs = self.size * y;
            for x in 0..(self.size - 1) {
                let x_offs = y_offs + x;

                indices.push(x_offs as u32);
                indices.push((x_offs + 1) as u32);
                indices.push((x_offs + self.size) as u32);

                indices.push((x_offs + 1) as u32);
                indices.push((x_offs + self.size + 1) as u32);
                indices.push((x_offs + self.size) as u32);
            }
        }

        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_vertices() {
        let data = vec![
            0.0, 1.0, 0.0,
            1.0, 2.0, 1.0,
            0.0, 1.0, 0.0,
        ];

        let heightmap = HeightMap::new(data, 3, 1.0);
        let verts = heightmap.vertices();

        assert_eq!(verts.len(), 9);

        assert_eq!(*verts.get(0).unwrap(), [0.0, 0.0, 0.0].into());
        assert_eq!(*verts.get(1).unwrap(), [1.0, -1.0, 0.0].into());
        assert_eq!(*verts.get(3).unwrap(), [0.0, -1.0, 1.0].into());

        assert_eq!(*verts.get(5).unwrap(), [2.0, -1.0, 1.0].into());
        assert_eq!(*verts.get(7).unwrap(), [1.0, -1.0, 2.0].into());
        assert_eq!(*verts.get(8).unwrap(), [2.0, 0.0, 2.0].into());
    }

    #[test]
    fn test_generate_indices() {
        let data = vec![
            0.0, 1.0, 0.0,
            1.0, 2.0, 1.0,
            0.0, 1.0, 0.0,
        ];

        let heightmap = HeightMap::new(data, 3, 1.0);
        let indices = heightmap.indices();

        assert_eq!(indices.len(), 24);

        assert_eq!(*indices.get(0).unwrap(), 0);
        assert_eq!(*indices.get(1).unwrap(), 1);
        assert_eq!(*indices.get(2).unwrap(), 3);

        assert_eq!(*indices.get(3).unwrap(), 1);
        assert_eq!(*indices.get(4).unwrap(), 4);
        assert_eq!(*indices.get(5).unwrap(), 3);
        
        assert_eq!(*indices.get(18).unwrap(), 4);
        assert_eq!(*indices.get(19).unwrap(), 5);
        assert_eq!(*indices.get(20).unwrap(), 7);

        assert_eq!(*indices.get(21).unwrap(), 5);
        assert_eq!(*indices.get(22).unwrap(), 8);
        assert_eq!(*indices.get(23).unwrap(), 7);
    }

    #[test]
    fn test_index_counts() {
        let data = vec![
            0.0, 1.0, 0.0,
            1.0, 2.0, 1.0,
            0.0, 1.0, 0.0,
        ];

        let heightmap = HeightMap::new(data, 3, 1.0);
        let indices = heightmap.indices();

        assert_eq!(indices.len(), 24);

        let data = vec![
            0.0, 1.0, 1.0, 0.0,
            1.0, 2.0, 2.0, 1.0,
            1.0, 2.0, 2.0, 1.0,
            0.0, 1.0, 1.0, 0.0,
        ];

        let heightmap = HeightMap::new(data, 4, 1.0);
        let indices = heightmap.indices();

        assert_eq!(indices.len(), 54);
    }
}