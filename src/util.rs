pub fn language(path: &std::path::PathBuf) -> Option<crate::codeptit::submit::SubmissionLanguage> {
    let extension = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();
    Some(match extension {
        "c" => crate::codeptit::submit::SubmissionLanguage::C,
        "cpp" => crate::codeptit::submit::SubmissionLanguage::Cpp,
        "python" => crate::codeptit::submit::SubmissionLanguage::Python,
        "cs" => crate::codeptit::submit::SubmissionLanguage::CSharp,
        "java" => crate::codeptit::submit::SubmissionLanguage::Java,
        _ => return None,
    })
}

pub fn question_code(
    language: &crate::codeptit::submit::SubmissionLanguage,
    code: &crate::codeptit::submit::SubmissionCode,
) -> anyhow::Result<Option<String>> {
    let (line_comment, block_comment_start, block_comment_end) = match language {
        crate::codeptit::submit::SubmissionLanguage::Python => ("#", "\"\"\"", "\"\"\""),
        _ => ("//", "/*", "*/"),
    };

    let (line, fallback) = match code {
        crate::codeptit::submit::SubmissionCode::File(path) => {
            let file = std::fs::File::open(path)?;
            let mut reader = std::io::BufReader::new(file);

            let mut line = String::new();
            std::io::BufRead::read_line(&mut reader, &mut line)?;

            let filename = path
                .file_stem()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or_default()
                .to_owned();
            (line, Some(filename))
        }
        crate::codeptit::submit::SubmissionCode::Source(source) => {
            (source.lines().next().unwrap_or_default().to_owned(), None)
        }
    };

    Ok(line
        .trim()
        .strip_prefix(line_comment)
        .or_else(|| {
            line.strip_prefix(block_comment_start)
                .and_then(|line| line.strip_suffix(block_comment_end))
        })
        .map(|line| line.trim().to_owned())
        .or(fallback))
}
