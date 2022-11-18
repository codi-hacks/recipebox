use std::io::BufRead;

use crate::common::header::HeaderMap;
use crate::parse::body::BodyParser;
use crate::parse::headers::HeadersParser;
use crate::parse::message::State::{Body, Finished, FirstLine, Headers};
use crate::parse::parse::{Parse, ParseResult};
use crate::parse::parse::ParseStatus::{Done, IoErr};

/// Generic HTTP message parser, used by both response and request parsing.
pub struct MessageParser<R, T> {
    read_body_if_no_content_length: bool,
    state: State<R, T>,
}

impl<R, T> MessageParser<R, T> {
    /// Creates a new message parser with the given parser to parse the first line.
    /// If read_body_if_no_content_length is true and no content length is provided, then the message
    /// body will consist of all data up to EOF. Otherwise the body will be empty.
    pub fn new(first_line_parser: R, read_body_if_no_content_length: bool) -> MessageParser<R, T> {
        MessageParser {
            state: FirstLine(first_line_parser),
            read_body_if_no_content_length,
        }
    }

    /// Gets the first line parser used by this message parser.
    /// May return None if the first line parser is no longer in use.
    pub fn first_line_parser(&self) -> Option<&R> {
        match &self.state {
            FirstLine(parser) => Some(parser),
            _ => None
        }
    }
}

/// The state of a message parser.
enum State<R, T> {
    /// Parsing the first line of the message.
    FirstLine(R),
    /// Parsing the headers.
    Headers(T, HeadersParser),
    /// Parsing the body.
    Body(T, HeaderMap, BodyParser),
    /// Parsing is complete.
    Finished(T, HeaderMap, Vec<u8>),
}

impl<T, R: Parse<T>> Parse<(T, HeaderMap, Vec<u8>)> for MessageParser<R, T> {
    fn parse(self, reader: &mut impl BufRead) -> ParseResult<(T, HeaderMap, Vec<u8>), Self> {
        let Self { mut state, read_body_if_no_content_length } = self;

        loop {
            let result = match state {
                FirstLine(parser) => first_line_state(reader, parser)?,
                Headers(first_line, parser) => headers_state(reader, first_line, parser, read_body_if_no_content_length)?,
                Body(first_line, headers, parser) => body_state(reader, first_line, headers, parser)?,
                Finished(first_line, headers, body) => return Ok(Done((first_line, headers, body)))
            };

            state = match result {
                Done(state) => state,
                IoErr(state, err) => return Ok(IoErr(Self { state, read_body_if_no_content_length }, err))
            }
        }
    }
}

/// Parses the first line and returns the next state if possible.
fn first_line_state<T, R: Parse<T>>(reader: &mut impl BufRead, parser: R) -> ParseResult<State<R, T>, State<R, T>> {
    Ok(match parser.parse(reader)? {
        Done(first_line) => Done(Headers(first_line, HeadersParser::new())),
        IoErr(parser, err) => IoErr(FirstLine(parser), err)
    })
}

/// Parses the headers and returns the next state if possible.
fn headers_state<T, R>(reader: &mut impl BufRead, first_line: T, parser: HeadersParser, read_body_if_no_content_length: bool) -> ParseResult<State<R, T>, State<R, T>> {
    Ok(match parser.parse(reader)? {
        Done(headers) => {
            let body_parser = BodyParser::new(&headers, read_body_if_no_content_length)?;
            Done(Body(first_line, headers, body_parser))
        }
        IoErr(parser, err) => IoErr(Headers(first_line, parser), err)
    })
}

/// Parses the body and returns the next state if possible.
fn body_state<T, R>(reader: &mut impl BufRead, first_line: T, headers: HeaderMap, parser: BodyParser) -> ParseResult<State<R, T>, State<R, T>> {
    Ok(match parser.parse(reader)? {
        Done(body) => Done(Finished(first_line, headers, body)),
        IoErr(parser, err) => IoErr(Body(first_line, headers, parser), err)
    })
}


#[cfg(test)]
mod tests {
    use std::io::{Error, ErrorKind};

    use crate::common::header::{CONTENT_LENGTH, Header, HeaderMap, HeaderMapOps, TRANSFER_ENCODING};
    use crate::header_map;
    use crate::parse::crlf_line::CrlfLineParser;
    use crate::parse::error::ParsingError::{BadSyntax, InvalidChunkSize, InvalidHeaderValue};
    use crate::parse::message::MessageParser;
    use crate::parse::test_util;
    use crate::parse::test_util::TestParseResult;
    use crate::parse::test_util::TestParseResult::{ParseErr, Value};

    type Message = (String, HeaderMap, Vec<u8>);
    type Parser = MessageParser<CrlfLineParser, String>;

    fn get_message_deframer(read_if_no_content_length: bool) -> Parser {
        MessageParser::new(CrlfLineParser::new(), read_if_no_content_length)
    }

    fn test_with_eof(input: Vec<&str>, read_if_no_content_length: bool, expected: TestParseResult<Message>) {
        test_util::test_with_eof(get_message_deframer(read_if_no_content_length), input, expected);
    }

    fn test_endless(data: Vec<&str>, endless_data: &str, read_if_no_content_length: bool, expected: TestParseResult<Message>) {
        test_util::test_endless_strs(get_message_deframer(read_if_no_content_length), data, endless_data, expected);
    }

    fn test_blocking(read_if_no_content_length: bool, tests: Vec<(Vec<&[u8]>, TestParseResult<Message>)>) {
        test_util::test_blocking(get_message_deframer(read_if_no_content_length), tests)
    }

    #[test]
    fn no_headers_or_body() {
        test_with_eof(
            vec!["blah blah blah\r\n\r\n"],
            false,
            Value(("blah blah blah".to_string(),
                   Default::default(),
                   vec![])),
        );
    }

    #[test]
    fn headers_and_body() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ncontent-length: 5\r\n\r\nhello"],
            false,
            Value(("HTTP/1.1 200 OK".to_string(),
                   HeaderMap::from_pairs(vec![(CONTENT_LENGTH, "5".to_string())]),
                   "hello".as_bytes().to_vec())),
        );
    }

    #[test]
    fn headers_and_body_fragmented() {
        test_with_eof(
            vec!["HTT", "P/1.", "1 200 OK", "\r", "\nconte", "nt-length", ":", " 5\r\n\r\nh", "el", "lo"],
            false,
            Value(("HTTP/1.1 200 OK".to_string(),
                   HeaderMap::from_pairs(vec![(CONTENT_LENGTH, "5".to_string())]),
                   "hello".as_bytes().to_vec())),
        );
    }

    #[test]
    fn only_one_message_returned() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ncontent-length: 5\r\n\r\nhello", "HTTP/1.1 200 OK\r\n\r\n", "HTTP/1.1 200 OK\r\n\r\n"],
            false,
            Value(("HTTP/1.1 200 OK".to_string(),
                   HeaderMap::from_pairs(vec![(CONTENT_LENGTH, "5".to_string())]),
                   "hello".as_bytes().to_vec())),
        );
    }

    #[test]
    fn big_body() {
        let body = b"iuwrhgiuelrguihwleriughwleiruhglweiurhgliwerg fkwfowjeofjiwoefijwef \
        wergiuwehrgiuwehilrguwehlrgiuw fewfwferg wenrjg; weirng lwieurhg owieurhg oeiuwrhg oewirg er\
        gweuirghweiurhgleiwurhglwieurhglweiurhglewiurhto8w374yto8374yt9p18234u50982@#$%#$%^&%^*(^)&(\
        *)_)+__+*()*()&**^%&$##!~!@~``12]\n3'\']\\l[.'\"lk]/l;<:?<:}|?L:|?L|?|:?e       oivj        \
        \n\n\n\n\\\t\t\t\t\t\t\t\\\t\t\t\t                                                          \
        ioerjgfoiaejrogiaergq34t2345123`    oijrgoi wjergi jweorgi jweorgji                 eworigj \
        riogj ewoirgj oewirjg 934598ut6932458t\ruyo3485gh o4w589ghu w458                          9ghu\
        pw94358gh pw93458gh pw9345gh pw9438g\rhu pw3945hg pw43958gh pw495gh :::;wefwefwef wef we  e ;;\
        @#$%@#$^@#$%&#$@%^#$%@#$%@$^%$&$%^*^%&(^$%&*#%^$&@$%^#!#$!~```~~~```wefwef wef ee f efefe e{\
        @#$%@#$^@#$%&#$@%^#$%@#$%@$^%$&$%^*^%&(^$%&*#%^$&@$%^#!#$!~```~~~```wefwef wef ee f efefe e{\
        @#$%@#$^@#$%&#$@%^#$%@#$%@$^%$&$%^*^%&(^$%&*#%^$&@$%^#!#$!~```~~~```wefwef wef ee f efefe e{\
        P{P[p[p[][][][]{}{}][][%%%\n\n\n\n\n\n wefwfw e2123456768960798676reresdsxfbcgrtg eg erg   ";
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ncontent-length: 1054\r\n\r\n", &String::from_utf8_lossy(body)],
            false,
            Value(("HTTP/1.1 200 OK".to_string(),
                   HeaderMap::from_pairs(vec![(CONTENT_LENGTH, "1054".to_string())]),
                   body.to_vec())),
        );
    }

    #[test]
    fn read_if_no_content_length_true() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\n\r\nhello", "HTTP/1.1 200 OK\r\n\r\n", "HTTP/1.1 200 OK\r\n\r\n"],
            true,
            Value(("HTTP/1.1 200 OK".to_string(),
                   Default::default(),
                   "helloHTTP/1.1 200 OK\r\n\r\nHTTP/1.1 200 OK\r\n\r\n".as_bytes().to_vec())),
        );
    }

    #[test]
    fn read_if_no_content_length_false() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\n\r\nhello", "HTTP/1.1 200 OK\r\n\r\n", "HTTP/1.1 200 OK\r\n\r\n"],
            false,
            Value(("HTTP/1.1 200 OK".to_string(),
                   Default::default(),
                   vec![])),
        );
    }

    #[test]
    fn custom_header() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ncustom-header: custom header value\r\n\r\n"],
            false,
            Value(("HTTP/1.1 200 OK".to_string(),
                   HeaderMap::from_pairs(vec![(Header::Custom("custom-header".to_string()), "custom header value".to_string())]),
                   vec![])),
        );
    }

    #[test]
    fn gibberish() {
        test_with_eof(
            vec!["ergejrogi jerogij eworfgjwoefjwof9wef wfw"],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }

    #[test]
    fn gibberish_with_newline() {
        test_with_eof(
            vec!["ergejrogi jerogij ewo\nrfgjwoefjwof9wef wfw"],
            false,
            ParseErr(BadSyntax),
        );
    }

    #[test]
    fn gibberish_with_crlf() {
        test_with_eof(
            vec!["ergejrogi jerogij ewo\r\nrfgjwoefjwof9wef wfw\r\n\r\n"],
            false,
            ParseErr(BadSyntax),
        );
    }

    #[test]
    fn gibberish_with_crlfs_at_end() {
        test_with_eof(
            vec!["ergejrogi jerogij eworfgjwoefjwof9wef wfw\r\n\r\n"],
            false,
            Value((
                "ergejrogi jerogij eworfgjwoefjwof9wef wfw".to_string(),
                Default::default(),
                vec![]
            )),
        );
    }

    #[test]
    fn all_newlines() {
        test_with_eof(
            vec!["\n\n\n\n\n\n\n\n\n\n\n"],
            false,
            ParseErr(BadSyntax),
        );
    }

    #[test]
    fn all_crlfs() {
        test_with_eof(
            vec!["\r\n\r\n\r\n\r\n"],
            false,
            Value(("".to_string(), Default::default(), vec![])),
        );
    }

    #[test]
    fn missing_crlfs() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK"],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }

    #[test]
    fn only_one_crlf() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\n"],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }

    #[test]
    fn bad_header() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\nbad header\r\n\r\n"],
            false,
            ParseErr(BadSyntax),
        );
    }

    #[test]
    fn bad_content_length_value() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ncontent-length: five\r\n\r\nhello"],
            false,
            ParseErr(InvalidHeaderValue),
        );
    }

    #[test]
    fn no_data() {
        test_with_eof(
            vec![],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }

    #[test]
    fn one_character() {
        test_with_eof(
            vec!["a"],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }

    #[test]
    fn one_crlf_nothing_else() {
        test_with_eof(
            vec!["\r\n"],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }

    #[test]
    fn content_length_too_long() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ncontent-length: 7\r\n\r\nhello"],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }

    #[test]
    fn content_length_too_long_with_request_after() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ncontent-length: 7\r\n\r\nhello", "HTTP/1.1 200 OK\r\n\r\n"],
            false,
            Value(("HTTP/1.1 200 OK".to_string(),
                   HeaderMap::from_pairs(vec![(CONTENT_LENGTH, "7".to_string())]),
                   "helloHT".as_bytes().to_vec())),
        );
    }

    #[test]
    fn content_length_too_short() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ncontent-length: 3\r\n\r\nhello"],
            false,
            Value(("HTTP/1.1 200 OK".to_string(),
                   HeaderMap::from_pairs(vec![(CONTENT_LENGTH, "3".to_string())]),
                   "hel".as_bytes().to_vec())),
        );
    }

    #[test]
    fn chunked_body() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "2\r\n",
                 "he\r\n",
                 "c\r\n",
                 "llo world he\r\n",
                 "3\r\n",
                 "llo\r\n",
                 "0\r\n",
                 "\r\n"],
            false,
            Value(("HTTP/1.1 200 OK".to_string(),
                   HeaderMap::from_pairs(vec![(TRANSFER_ENCODING, "chunked".to_string())]),
                   "hello world hello".as_bytes().to_vec())),
        );
    }

    #[test]
    fn chunked_body_no_termination() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "2\r\n",
                 "he\r\n",
                 "c\r\n",
                 "llo world he\r\n",
                 "3\r\n",
                 "llo\r\n"],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }

    #[test]
    fn chunked_body_chunk_size_1_byte_too_large() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "3\r\n",
                 "he\r\n",
                 "c\r\n",
                 "llo world he\r\n",
                 "3\r\n",
                 "llo\r\n",
                 "0\r\n",
                 "\r\n"],
            false,
            ParseErr(BadSyntax),
        );
    }

    #[test]
    fn chunked_body_chunk_size_2_bytes_too_large() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "4\r\n",
                 "he\r\n",
                 "c\r\n",
                 "llo world he\r\n",
                 "3\r\n",
                 "llo\r\n",
                 "0\r\n",
                 "\r\n"],
            false,
            ParseErr(BadSyntax),
        );
    }

    #[test]
    fn chunked_body_chunk_size_many_bytes_too_large() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "13\r\n",
                 "he\r\n",
                 "c\r\n",
                 "llo world he\r\n",
                 "3\r\n",
                 "llo\r\n",
                 "0\r\n",
                 "\r\n"],
            false,
            Value(("HTTP/1.1 200 OK".to_string(),
                   HeaderMap::from_pairs(vec![(TRANSFER_ENCODING, "chunked".to_string())]),
                   "he\r\nc\r\nllo world hello".as_bytes().to_vec())),
        );
    }

    #[test]
    fn chunked_body_huge_chunk_size() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "100\r\n",
                 "he\r\n",
                 "c\r\n",
                 "llo world he\r\n",
                 "3\r\n",
                 "llo\r\n",
                 "0\r\n",
                 "\r\n"],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }

    #[test]
    fn chunked_body_chunk_size_not_hex_digit() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "z\r\n",
                 "he\r\n",
                 "c\r\n",
                 "llo world he\r\n",
                 "3\r\n",
                 "llo\r\n",
                 "0\r\n",
                 "\r\n"],
            false,
            ParseErr(InvalidChunkSize),
        );
    }

    #[test]
    fn chunked_body_no_crlfs() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "zhelloiouf jwiufji ejif jef"],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }


    #[test]
    fn chunked_body_no_content() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "9\r\n",
                 "\r\n"],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }

    #[test]
    fn chunked_body_no_trailing_crlf() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "2\r\n",
                 "he\r\n",
                 "c\r\n",
                 "llo world he\r\n",
                 "3\r\n",
                 "llo\r\n",
                 "0\r\n"],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }

    #[test]
    fn chunked_body_only_chunk_size() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "2\r\n",
                 "he"],
            false,
            ErrorKind::UnexpectedEof.into(),
        );
    }

    #[test]
    fn empty_chunked_body() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "0\r\n",
                 "\r\n"],
            false,
            Value(("HTTP/1.1 200 OK".to_string(),
                   HeaderMap::from_pairs(vec![(TRANSFER_ENCODING, "chunked".to_string())]),
                   vec![])),
        );
    }

    #[test]
    fn chunked_body_huge_chunk() {
        let chunk = "eofjaiweughlwauehgliw uehfwaiuefhpqiwuefh lwieufh wle234532\
                 57rgoi jgoai\"\"\"woirjgowiejfiuf hawlieuf halweifu hawef awef \
                 weFIU HW iefu\t\r\n\r\nhweif uhweifuh qefq234523 812u9405834205 \
                 8245 1#@%^#$*&&^(*&)()&%^$%#^$]\r;g]ew r;g]ege\n\r\n\r\noweijf ow\
                 aiejf; aowiejf owf ifoa iwf aioerjf aoiwerjf laiuerwhgf lawiuefhj owfjdc\
                  wf                 awefoi jwaeoif jwei          WEAOFIJ AOEWI FJA EFJ  few\
                  wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                  weiofj weoifj oweijfo qwiejfo quehfow uehfo qiwjfpo qihw fpqeighpqf efoiwej foq\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                                  weiofj weoifj oweijfo qwiejfo quehfow uehfo qiwjfpo qihw fpqeighpqf efoiwej foq\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                                  weiofj weoifj oweijfo qwiejfo quehfow uehfo qiwjfpo qihw fpqeighpqf efoiwej foq\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                                  weiofj weoifj oweijfo qwiejfo quehfow uehfo qiwjfpo qihw fpqeighpqf efoiwej foq\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                                  weiofj weoifj oweijfo qwiejfo quehfow uehfo qiwjfpo qihw fpqeighpqf efoiwej foq\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                                  weiofj weoifj oweijfo qwiejfo quehfow uehfo qiwjfpo qihw fpqeighpqf efoiwej foq\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowi\r\nefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj ae\r\nlirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj ae\nlirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf\n oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowi\nefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
                 wefoi jawoiefj aowiefgj aelirugh aliowefj oaweijf oweijf owiejf oweifj weof\
         ";
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n",
                 "C2A\r\n",
                 chunk,
                 "\r\n",
                 "0\r\n",
                 "\r\n"],
            false,
            Value(("HTTP/1.1 200 OK".to_string(),
                   HeaderMap::from_pairs(vec![(TRANSFER_ENCODING, "chunked".to_string())]),
                   chunk.as_bytes().to_vec())),
        );
    }

    #[test]
    fn huge_first_line() {
        test_with_eof(
            vec!["HTTP/1.1 200 OKroig jseorgi jpseoriegj seorigj epoirgj epsigrj paweorgj aeo\
            6rgj seprogj aeorigj pserijg pseirjgp seijg aowijrg03w8u4 t0q83u40 qwifwagf awiorjgf aowi\
            4rgj seprogj aeorigj pserijg pseirjgp seijg aowijrg03w8u4 t0q83u40 qwifwagf awiorjgf aowi\
            3rgj seprogj aeorigj pserijg pseirjgp seijg aowijrg03w8u4 t0q83u40 qwifwagf awiorjgf aowi\
            2rgj seprogj aeorigj pserijg pseirjgp seijg aowijrg03w8u4 t0q83u40 qwifwagf awiorjgf aowi\
            1rgj seprogj aeorigj pserijg pseirjgp seijg aowijrg03w8u4 t0q83u40 qwifwagf awiorjgf aowi\
            4rgj seprogj aeorigj pserijg pseirjgp seijg aowijrg03w8u4 t0q83u40 qwifwagf awiorjgf aowi\
            8rgj seprogj aeorigj pserijg pseirjgp seijg aowijrg03w8u4 t0q83u40 qwifwagf awiorjgf aowi\
            9fj asodijv osdivj osidvja psijf pasidjf pas\r\n\
            content-length: 5\r\n\r\nhello"],
            false,
            Error::new(ErrorKind::Other, "read limit reached").into(),
        );
    }

    #[test]
    fn huge_header() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\n",
                 "big-header: iowjfo iawjeofiajw pefiawjpefoi hwjpeiUF HWPIU4FHPAIWUHGPAIWUHGP AIWUHGRP \
            9Q43GHP 9Q3824U P9 658 23 YP 5698U24P985U2P198 4YU5P23985THPWERIUHG LIEAHVL DIFSJNV LAID\
            9Q43GHP 9Q3824U P9 658 23 YP 5698U24P985U2P198 4YU5P23985THPWERIUHG LIEAHVL DIFSJNV LAID\
            9Q43GHP 9Q3824U P9 658 23 YP 5698U24P985U2P198 4YU5P23985THPWERIUHG LIEAHVL DIFSJNV LAID\
            9Q43GHP 9Q3824U P9 658 23 YP 5698U24P985U2P198 4YU5P23985THPWERIUHG LIEAHVL DIFSJNV LAID\
            9Q43GHP 9Q3824U P9 658 23 YP 5698U24P985U2P198 4YU5P23985THPWERIUHG LIEAHVL DIFSJNV LAID\
            9Q43GHP 9Q3824U P9 658 23 YP 5698U24P985U2P198 4YU5P23985THPWERIUHG LIEAHVL DIFSJNV LAID\
            3JFHVL AIJFHVL AILIHiuh waiufh iefuhapergiu hapergiu hapeirug haeriug hsperg ",
                 "\r\n\r\n"],
            false,
            Error::new(ErrorKind::Other, "read limit reached").into(),
        );
    }

    #[test]
    fn zero_content_length() {
        test_with_eof(
            vec!["HTTP/1.1 200 OK\r\n",
                 "content-length: 0\r\n",
                 "\r\n"],
            false,
            Value(("HTTP/1.1 200 OK".to_string(), header_map![(CONTENT_LENGTH, "0")], vec![])),
        );
    }

    #[test]
    fn endless_line() {
        test_endless(
            vec![],
            "blah",
            false,
            Error::new(ErrorKind::Other, "read limit reached").into(),
        )
    }

    #[test]
    fn endless_headers() {
        test_endless(
            vec!["HTTP/1.1 200 OK\r\n"],
            "random: blah\r\n",
            false,
            Error::new(ErrorKind::Other, "read limit reached").into(),
        );

        test_endless(
            vec!["HTTP/1.1 200 OK\r\n"],
            "random: blahh\r\n",
            false,
            Error::new(ErrorKind::Other, "read limit reached").into(),
        );

        test_endless(
            vec!["HTTP/1.1 200 OK\r\n"],
            "random: blahhhh\r\n",
            false,
            Error::new(ErrorKind::Other, "read limit reached").into(),
        );

        test_endless(
            vec!["HTTP/1.1 200 OK\r\n"],
            "a: a\r\n",
            false,
            Error::new(ErrorKind::Other, "read limit reached").into(),
        );

        test_endless(
            vec!["HTTP/1.1 200 OK\r\n"],
            "a",
            false,
            Error::new(ErrorKind::Other, "read limit reached").into(),
        );

        test_endless(
            vec!["HTTP/1.1 200 OK\r\n"],
            "a: ",
            false,
            Error::new(ErrorKind::Other, "read limit reached").into(),
        );

        test_endless(
            vec!["HTTP/1.1 200 OK\r\n"],
            ": ",
            false,
            Error::new(ErrorKind::Other, "read limit reached").into(),
        );
    }

    #[test]
    fn endless_body() {
        test_endless(
            vec!["HTTP/1.1 200 OK\r\n\r\n"],
            "blah blah blah",
            true,
            Error::new(ErrorKind::Other, "read limit reached").into(),
        )
    }

    #[test]
    fn blocking_with_headers_and_body() {
        test_blocking(false, vec![
            (vec![b"GET ", b"/so"], ErrorKind::WouldBlock.into()),
            (vec![], ErrorKind::WouldBlock.into()),
            (vec![b"mera", b"ndomurl "], ErrorKind::WouldBlock.into()),
            (vec![], ErrorKind::WouldBlock.into()),
            (vec![b"HT"], ErrorKind::WouldBlock.into()),
            (vec![b"T"], ErrorKind::WouldBlock.into()),
            (vec![], ErrorKind::WouldBlock.into()),
            (vec![], ErrorKind::WouldBlock.into()),
            (vec![b"P"], ErrorKind::WouldBlock.into()),
            (vec![b"/1.1"], ErrorKind::WouldBlock.into()),
            (vec![b"\r"], ErrorKind::WouldBlock.into()),
            (vec![b"\n"], ErrorKind::WouldBlock.into()),
            (vec![b"hell"], ErrorKind::WouldBlock.into()),
            (vec![b"o: "], ErrorKind::WouldBlock.into()),
            (vec![b"value\r\n"], ErrorKind::WouldBlock.into()),
            (vec![], ErrorKind::WouldBlock.into()),
            (vec![b"con", b"te", b"nt", b"-length: 5"], ErrorKind::WouldBlock.into()),
            (vec![], ErrorKind::WouldBlock.into()),
            (vec![b"\r\n"], ErrorKind::WouldBlock.into()),
            (vec![], ErrorKind::WouldBlock.into()),
            (vec![b"\r\n"], ErrorKind::WouldBlock.into()),
            (vec![b"h"], ErrorKind::WouldBlock.into()),
            (vec![b"e"], ErrorKind::WouldBlock.into()),
            (vec![], ErrorKind::WouldBlock.into()),
            (vec![b"ll"], ErrorKind::WouldBlock.into()),
            (vec![b"o"], Value(("GET /somerandomurl HTTP/1.1".to_string(), header_map![("content-length", "5"), ("hello", "value")], b"hello".to_vec()))),
        ])
    }
}