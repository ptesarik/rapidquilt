impl<Line> UnifiedPatchHunkHeaderWriter for Hunk<'_, Line> {
        writer.write_all(NULL_FILENAME)?;
        writer.write_all(NULL_FILENAME)?;
        writer.write_all(self.header)?;