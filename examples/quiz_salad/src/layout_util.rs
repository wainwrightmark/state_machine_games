#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spacing {
    SpaceBetween,
    SpaceAround,
    Centre,
}

impl Spacing {
    pub const fn apply(
        &self,
        parent_ideal_length: f32,
        child_ideal_length: f32,
        num_children: f32,
        child_index: f32,
    ) -> f32 {
        let total_padding = parent_ideal_length - (num_children * child_ideal_length);

        match self {
            Spacing::SpaceBetween => {
                if num_children == 0.0 {
                    0.0
                } else if num_children == 1.0 {
                    total_padding / 2.0
                } else {
                    let padding_between_children = total_padding / (num_children - 1.0);
                    (padding_between_children + child_ideal_length) * child_index
                }
            }
            Spacing::SpaceAround => {
                if num_children == 0.0 {
                    0.0
                } else {
                    let left_or_right_padding = total_padding / (num_children as f32 * 2.);

                    let paddings = 1.0 + (child_index * 2.0);

                    (paddings * left_or_right_padding) + (child_index * child_ideal_length)
                }
            }
            Spacing::Centre => {
                let top_padding = total_padding / 2.;
                top_padding + (child_index as f32 * child_ideal_length)
            }
        }
    }
}
