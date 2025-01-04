use crate::github::{CommentData, IssueData};
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Markdown,
    Xml,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "xml" => Ok(OutputFormat::Xml),
            _ => Err(format!("Unsupported format: {}", s)),
        }
    }
}

pub trait FormatWriter {
    fn write_issue(&self, issue: &IssueData, comments: &[CommentData]) -> String;
}

pub struct MarkdownWriter;

impl FormatWriter for MarkdownWriter {
    fn write_issue(&self, issue: &IssueData, comments: &[CommentData]) -> String {
        let mut content = format!(
            "# Issue #{}: {}\n\n\
             **State:** {}\n\
             **Created:** {}\n\
             **Updated:** {}\n\
             **Labels:** {}\n\
             **Assignees:** {}\n\
             **User:** {}\n\n\
             ## Description\n\n\
             {}\n",
            issue.number(),
            issue.title(),
            issue.state(),
            issue.created_at(),
            issue.updated_at(),
            issue.labels().join(", "),
            issue.assignees().join(", "),
            issue.user(),
            issue.body().unwrap_or("*No description provided*")
        );

        if !comments.is_empty() {
            content.push_str("\n## Comments\n\n");
            for comment in comments {
                content.push_str(&format!(
                    "### @{} ({})\n\n{}\n\n",
                    comment.user, comment.created_at, comment.body
                ));
            }
        }

        content
    }
}

pub struct XmlWriter;

impl FormatWriter for XmlWriter {
    fn write_issue(&self, issue: &IssueData, comments: &[CommentData]) -> String {
        let mut content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<issue>
    <number>{}</number>
    <title><![CDATA[{}]]></title>
    <state>{}</state>
    <created_at>{}</created_at>
    <updated_at>{}</updated_at>
    <labels>
        {}
    </labels>
    <assignees>
        {}
    </assignees>
    <user>{}</user>
    <description><![CDATA[
        {}
    ]]></description>"#,
            issue.number(),
            issue.title(),
            issue.state(),
            issue.created_at(),
            issue.updated_at(),
            issue
                .labels()
                .iter()
                .map(|l| format!("        <label>{}</label>", l))
                .collect::<Vec<_>>()
                .join("\n"),
            issue
                .assignees()
                .iter()
                .map(|a| format!("        <assignee>{}</assignee>", a))
                .collect::<Vec<_>>()
                .join("\n"),
            issue.user(),
            issue.body().unwrap_or("No description provided"),
        );

        if !comments.is_empty() {
            content.push_str("\n    <comments>\n");
            for comment in comments {
                content.push_str(&format!(
                    r#"        <comment>
            <user>{}</user>
            <created_at>{}</created_at>
            <body><![CDATA[{}]]></body>
        </comment>
"#,
                    comment.user, comment.created_at, comment.body
                ));
            }
            content.push_str("    </comments>\n");
        }

        content.push_str("</issue>\n");
        content
    }
}

pub fn get_writer(format: OutputFormat) -> Box<dyn FormatWriter> {
    match format {
        OutputFormat::Markdown => Box::new(MarkdownWriter),
        OutputFormat::Xml => Box::new(XmlWriter),
    }
}

pub fn get_file_extension(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Markdown => "md",
        OutputFormat::Xml => "xml",
    }
}
