use ws_core::Ustr;

pub fn split_two_line_ustr(text: Ustr, max_line_len: usize) -> itertools::Either<Ustr, (Ustr, Ustr)> {
    if text.len() > max_line_len {
        let half_len = text.len() / 2;

        if let Some((index, _)) = text
            .char_indices()
            .filter(|(_, c)| c.is_whitespace())
            .min_by_key(|(i, _c)| i.abs_diff(half_len))
        {
            let (start, end) = text.split_at(index);
            let (_, end) = end.split_at(1);

            itertools::Either::Right((Ustr::from(start), Ustr::from(end)))
        } else {
            itertools::Either::Left(text)
        }
    } else {
        itertools::Either::Left(text)
    }
}