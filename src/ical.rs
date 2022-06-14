use chrono::prelude::*;

struct ContentLine<'a> {
    name: std::borrow::Cow<'a, str>,
    params: std::borrow::Cow<'a, [Parameter<'a>]>,
    value: std::borrow::Cow<'a, str>
}

fn escape_text(param: &str) -> String {
    param
        .replace('\\', "\\\\")
        .replace(";", "\\;")
        .replace(",", "\\,")
        .replace("\n", "\\n")
}

impl std::fmt::Display for ContentLine<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = format!("{}{}:{}", self.name, self.params.iter().map(ToString::to_string).fold(String::new(), |accum, i| {
            format!("{};{}", accum, i)
        }), escape_text(&self.value));

        let mut lines = vec![];
        let mut cur_line = String::new();
        let mut a = 0;
        let mut last_i = 0;
        for (i, c) in line.char_indices() {
            let s = i - last_i;
            if a + s < 75 {
                cur_line.push(c);
                a += s;
            } else {
                lines.push(cur_line);
                cur_line = format!(" {}", c);
                a = s;
            }
            last_i = i;
        }
        lines.push(cur_line);

        for line in lines {
            write!(f, "{}\r\n", line)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Parameter<'a> {
    name: std::borrow::Cow<'a, str>,
    values: std::borrow::Cow<'a, [std::borrow::Cow<'a, str>]>
}

fn escape_param<'a>(param: &'a std::borrow::Cow<'a, str>) -> std::borrow::Cow<'a, str> {
    if param.chars().any(|c| c == ':' || c == ';' || c == ',') {
        return std::borrow::Cow::Owned(format!("\"{}\"", param));
    } else {
        return std::borrow::Cow::Borrowed(&param);
    }
}

impl std::fmt::Display for Parameter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.values.into_iter().map(escape_param).fold(String::new(), |accum, i| {
            if accum.is_empty() {
                i.to_string()
            } else {
                format!("{},{}", accum, i)
            }
        }))
    }
}

fn format_datetime(date_time: &DateTime<Utc>) -> String {
    date_time.format("%Y%m%dT%H%M%SZ").to_string()
}

pub struct Calendar {
    pub product: String,
    pub version: String,
    pub scale: Option<String>,
    pub method: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub uid: Option<String>,
    pub url: Option<String>,
    pub events: Vec<Event>,
}

impl Calendar {
    fn to_content_lines<'a>(&'a self) -> Vec<ContentLine<'a>> {
        let mut out = vec![];
        out.push(ContentLine {
            name: std::borrow::Cow::Borrowed("BEGIN"),
            params: std::borrow::Cow::Borrowed(&[]),
            value: std::borrow::Cow::Borrowed("VCALENDAR")
        });
        out.push(ContentLine {
            name: std::borrow::Cow::Borrowed("PRODID"),
            params: std::borrow::Cow::Borrowed(&[]),
            value: std::borrow::Cow::Borrowed(&self.product)
        });
        out.push(ContentLine {
            name: std::borrow::Cow::Borrowed("VERSION"),
            params: std::borrow::Cow::Borrowed(&[]),
            value: std::borrow::Cow::Borrowed(&self.version)
        });
        if let Some(scale) = &self.scale {
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("CALSCALE"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Borrowed(scale)
            });
        }
        if let Some(method) = &self.method {
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("METHOD"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Borrowed(method)
            });
        }
        if let Some(name) = &self.name {
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("NAME"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Borrowed(name)
            });
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("X-WR-CALNAME"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Borrowed(name)
            });
        }
        if let Some(description) = &self.description {
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("DESCRIPTION"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Borrowed(description)
            });
        }
        if let Some(uid) = &self.uid {
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("UID"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Borrowed(uid)
            });
        }
        if let Some(url) = &self.url {
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("URL"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Borrowed(url)
            });
        }
        for event in &self.events {
            out.extend(event.to_content_lines());
        }
        out.push(ContentLine {
            name: std::borrow::Cow::Borrowed("END"),
            params: std::borrow::Cow::Borrowed(&[]),
            value: std::borrow::Cow::Borrowed("VCALENDAR")
        });
        out
    }
}

impl std::fmt::Display for Calendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.to_content_lines() {
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

pub struct Organiser {
    pub address: String,
    pub common_name: Option<String>,
    pub sent_by: Option<String>
}

impl Organiser {
    fn to_content_line<'a>(&'a self) -> ContentLine<'a> {
        let mut params = vec![];
        if let Some(common_name) = &self.common_name {
            params.push(Parameter {
                name: std::borrow::Cow::Borrowed("CN"),
                values: std::borrow::Cow::Owned(vec![std::borrow::Cow::Borrowed(common_name.as_str())])
            });
        }
        if let Some(sent_by) = &self.sent_by {
            params.push(Parameter {
                name: std::borrow::Cow::Borrowed("SENT-BY"),
                values: std::borrow::Cow::Owned(vec![std::borrow::Cow::Borrowed(sent_by.as_str())])
            });
        }
        ContentLine {
            name: std::borrow::Cow::Borrowed("ORGANIZER"),
            params: std::borrow::Cow::Owned(params),
            value: std::borrow::Cow::Borrowed(&self.address)
        }
    }
}

#[allow(dead_code)]
pub enum Image {
    Url(String),
    Binary(Vec<u8>)
}

impl Image {
    fn to_content_line<'a>(&'a self) -> ContentLine<'a> {
        match self {
            Self::Url(url) => {
                ContentLine {
                    name: std::borrow::Cow::Borrowed("IMAGE"),
                    params: std::borrow::Cow::Owned(vec![Parameter {
                        name: std::borrow::Cow::Borrowed("VALUE"),
                        values: std::borrow::Cow::Owned(vec![std::borrow::Cow::Borrowed("URI")])
                    }]),
                    value: std::borrow::Cow::Borrowed(url)
                }
            }
            Self::Binary(data) => {
                ContentLine {
                    name: std::borrow::Cow::Borrowed("IMAGE"),
                    params: std::borrow::Cow::Owned(vec![Parameter {
                        name: std::borrow::Cow::Borrowed("VALUE"),
                        values: std::borrow::Cow::Owned(vec![std::borrow::Cow::Borrowed("BINARY")])
                    }, Parameter {
                        name: std::borrow::Cow::Borrowed("ENCODING"),
                        values: std::borrow::Cow::Owned(vec![std::borrow::Cow::Borrowed("BASE64")])
                    }]),
                    value: std::borrow::Cow::Owned(base64::encode(data))
                }
            }
        }
    }
}

pub struct Event {
    pub uid: String,
    pub timestamp: DateTime<Utc>,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
    pub created: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub summary: Option<String>,
    pub location: Option<String>,
    pub organiser: Option<Organiser>,
    pub status: Option<String>,
    pub images: Vec<Image>
}

impl Event {
    fn to_content_lines<'a>(&'a self) -> Vec<ContentLine<'a>> {
        let mut out = vec![];
        out.push(ContentLine {
            name: std::borrow::Cow::Borrowed("BEGIN"),
            params: std::borrow::Cow::Borrowed(&[]),
            value: std::borrow::Cow::Borrowed("VEVENT")
        });
        out.push(ContentLine {
            name: std::borrow::Cow::Borrowed("UID"),
            params: std::borrow::Cow::Borrowed(&[]),
            value: std::borrow::Cow::Borrowed(&self.uid)
        });
        out.push(ContentLine {
            name: std::borrow::Cow::Borrowed("DTSTAMP"),
            params: std::borrow::Cow::Borrowed(&[]),
            value: std::borrow::Cow::Owned(format_datetime(&self.timestamp))
        });
        out.push(ContentLine {
            name: std::borrow::Cow::Borrowed("DTSTART"),
            params: std::borrow::Cow::Borrowed(&[]),
            value: std::borrow::Cow::Owned(format_datetime(&self.start))
        });
        if let Some(end) = &self.end {
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("DTEND"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Owned(format_datetime(end))
            });
        }
        if let Some(created) = &self.created {
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("CREATED"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Owned(format_datetime(created))
            });
        }
        if let Some(description) = &self.description {
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("DESCRIPTION"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Borrowed(description)
            });
        }
        if let Some(summary) = &self.summary {
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("SUMMARY"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Borrowed(summary)
            });
        }
        if let Some(location) = &self.location {
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("LOCATION"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Borrowed(location)
            });
        }
        if let Some(organiser) = &self.organiser {
            out.push(organiser.to_content_line());
        }
        if let Some(status) = &self.status {
            out.push(ContentLine {
                name: std::borrow::Cow::Borrowed("STATUS"),
                params: std::borrow::Cow::Borrowed(&[]),
                value: std::borrow::Cow::Borrowed(status)
            });
        }
        for image in &self.images {
            out.push(image.to_content_line());
        }
        out.push(ContentLine {
            name: std::borrow::Cow::Borrowed("END"),
            params: std::borrow::Cow::Borrowed(&[]),
            value: std::borrow::Cow::Borrowed("VEVENT")
        });
        out
    }
}