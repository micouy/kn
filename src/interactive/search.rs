    pub fn consume_char(
        &mut self,
        c: char,
    ) -> (Vec<String>, String, Vec<Entry>) {
        let findings = if c == '/' {
            if !self.input.is_empty() {
                // Perhaps repeating the search is unnecessary. It would
                // be enough to cache the previous search and just push it to
                // findings.
                let input = mem::replace(&mut self.input, String::new());
                let abbr = Abbr::from_string(input).unwrap();

                // Get matching entries and order them.
                let mut entries: Vec<_> = self
                    .current_level
                    .iter()
                    .filter_map(|entry| match entry.advance(&abbr) {
                        Flow::DeadEnd => None,
                        Flow::Continue(entry) => Some(entry),
                    })
                    .collect();
                entries.sort_by(|a, b| a.congruence.cmp(&b.congruence));

                // Fill current level with children of the previous one.
                self.current_level.clear();
                let engine = &self.engine;
                self.current_level.extend(
                    entries
                        .iter()
                        .map(|Entry { path, congruence }| {
                            engine.read_dir(path).into_iter().map(move |path| {
                                Entry {
                                    path,
                                    congruence: congruence.clone(),
                                }
                            })
                        })
                        .flatten(),
                );

                self.findings.push(Finding { abbr, entries });
            }

            self.findings
                .last()
                .map(|Finding { entries, .. }| entries.clone())
                .unwrap_or_else(|| vec![])
        } else {
            // Construct a new abbr.
            self.input.push(c);
            let abbr = Abbr::from_string(self.input.clone()).unwrap();

            // Get matching entries and order them.
            let mut entries: Vec<_> = self
                .current_level
                .iter()
                .filter_map(|entry| match entry.advance(&abbr) {
                    Flow::DeadEnd => None,
                    Flow::Continue(entry) => Some(entry),
                })
                .collect();
            entries.sort_by(|a, b| a.congruence.cmp(&b.congruence));

            entries
        };

        let location = self
            .findings
            .iter()
            .filter_map(|Finding { entries, .. }| entries.get(0))
            .filter_map(|Entry { path, .. }| path.file_name())
            .map(|file_name| file_name.to_string_lossy().to_string())
            .collect();

        (location, self.input.clone(), findings)
    }

