#![allow(dead_code)]

extern crate rustc_serialize;
extern crate hyper;
extern crate chrono;

use std::collections::btree_map::IntoIter;
use std::collections::BTreeMap;
use std::io::Read;

use hyper::client::Client;
use hyper::header::{UserAgent, LastModified, HttpDate};
use hyper::error::Error as HyperError;

use rustc_serialize::json;
use rustc_serialize::json::DecoderError;

fn setup_post(board: String, native: NativePost) -> FourchanPost {
    //println!("{:?}", native.clone());
    FourchanPost {
        _board: board.clone(),
        _no: native.no,
        _op: if native.resto == 0 {
            OpStatus::Yes
        } else {
            OpStatus::No(native.resto)
        },
        _sticky: native.sticky.is_some(),
        _closed: native.closed.is_some(),
        _timestamp: native.time,
        _name: native.name,
        _trip: native.trip,
        _id: native.id,
        _capcode: native.capcode,
        _country: if native.country.is_some() {
            Some([native.country.clone().unwrap().chars().nth(0).unwrap(), native.country.unwrap().chars().nth(1).unwrap()])
        } else {
            None
        },
        _subject: match native.sub {
            None => "".to_owned(),
            Some(res) => res,
        },
        _comment: match native.com {
            None => "".to_owned(),
            Some(res) => res,
        },
        _files: {
            let mut files: Vec<FourchanFile> = Vec::new();
            if native.ext.is_some() {
                files.push(FourchanFile {
                    _board: board,
                    _renamed: native.tim.unwrap().to_string(),
                    _filename: native.filename.unwrap(),
                    _extension: native.ext.unwrap(),
                    _size: native.fsize.unwrap(),
                    _md5: native.md5.unwrap(),
                    _w: native.w.unwrap(),
                    _h: native.h.unwrap(),
                    _thumb_w: native.tn_w.unwrap(),
                    _thumb_h: native.tn_h.unwrap(),
                    _deleted: if native.filedeleted.is_none() {
                        false
                    } else {
                        native.filedeleted.unwrap() > 0
                    },
                    _spoiler: if native.filedeleted.is_none() {
                        false
                    } else {
                        native.spoiler.unwrap() > 0
                    },
                    _custom_spoiler: native.custom_spoiler
                });
            }
            files
        }
    }
}

pub trait HasUrl {
    fn url(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum OpStatus {
    Yes,
    No(u64),
}

#[derive(Debug)]
pub enum UrlType {
    Full,
    Thumb,
}

#[derive(Debug)]
pub enum FilenameType {
    Renamed,
    Original,
}

#[derive(Debug)]
pub enum ChanError {
    JSON(DecoderError),
    HTTP(HyperError),
    NonexistantBoard(String),
}

#[derive(Debug)]
pub enum SpoilerType {
    Yes(Option<u8>),
    No,
}

#[derive(Debug)]
pub struct Fourchan {
    _boards: BTreeMap<String, FourchanBoard>
}

#[derive(Debug, Clone)]
pub struct FourchanBoard {
    _shortcode: String, 
    _title: String, 
    _work_safe: bool, 
    _per_page: u8, 
    _pages: u8, 
    _using_catalog: bool, 
    _empty: bool, 
    _threads: Vec<FourchanThread>,
}

#[derive(Debug, Clone)]
pub struct FourchanThread {
    _op: Option<FourchanPost>, 
    _board: String, 
    _no: u64, 
    _last_modified: Option<u32>, 
    _posts: Vec<FourchanPost>, 
    _empty: bool,
}

#[derive(Debug, Clone)]
pub struct FourchanPost {
    _board: String, 
    _no: u64, 
    _op: OpStatus, 
    _sticky: bool, 
    _closed: bool, 
    _timestamp: u32, 
    _name: Option<String>, 
    _trip: Option<String>, 
    _id: Option<String>,
    _capcode: Option<String>,
    _country: Option<[char; 2]>,
    _subject: String,
    _comment: String,
    _files: Vec<FourchanFile>,
}

#[derive(Debug, Clone)]
pub struct FourchanFile {
    _board: String,
    _renamed: String,
    _filename: String,
    _extension: String,
    _size: u32,
    _md5: String,
    _w: u16,
    _h: u16,
    _thumb_w: u8,
    _thumb_h: u8,
    _deleted: bool,
    _spoiler: bool,
    _custom_spoiler: Option<u8>,
}

// will panic if an http request is weird. TODO: add a match at the .unwrap()
fn download(url: &str) -> Result<(String, Option<u64>), HyperError> { // TODO: have functions actually do something with the timestamp
    let res = Client::new().get(url).header(UserAgent(env!("CARGO_PKG_NAME").to_owned() + "/v" + env!("CARGO_PKG_VERSION"))).send();
    let mut output = String::new();
    match res {
        Err(e) => Err(e),
        Ok(mut val) => {
            let _ = val.read_to_string(&mut output);
            //println!("{:?}", val.headers.get::<LastModified>().unwrap().to_timespec().sec);
            //println!("{} - {:?}", url, );
            Ok((output, match val.headers.get::<LastModified>() {
                None => None,
                Some(&LastModified(time)) => match time {
                    HttpDate(time) => Some(time.to_timespec().sec as u64),
                },
            }))
        },
    }
}

#[derive(RustcDecodable, Debug, Clone)]
pub struct NativePost {
    pub no: u64,
    pub resto: u64,
    pub sticky: Option<u8>,
    pub closed: Option<u8>,
    //pub archived: Option<u8>, // is this useful for us at this time?
    pub now: String,
    pub time: u32,
    pub name: Option<String>,
    pub trip: Option<String>,
    pub id: Option<String>,
    pub capcode: Option<String>,
    pub country: Option<String>,
    //pub country_name: Option<String>,
    pub sub: Option<String>,
    pub com: Option<String>,
    pub tim: Option<u64>,
    pub filename: Option<String>,
    pub ext: Option<String>,
    pub fsize: Option<u32>,
    pub md5: Option<String>,
    pub w: Option<u16>,
    pub h: Option<u16>,
    pub tn_w: Option<u8>,
    pub tn_h: Option<u8>,
    pub filedeleted: Option<u8>,
    pub spoiler: Option<u8>,
    pub custom_spoiler: Option<u8>,
    pub replies: Option<u32>,
    pub images: Option<u32>,
    pub bumplimit: Option<u8>,
    pub imagelimit: Option<u8>,
    pub last_modified: Option<u32>,
}

#[derive(RustcDecodable, Debug)]
pub struct NativeThread {
    pub posts: Vec<NativePost>
}

#[derive(RustcDecodable, Debug)]
pub struct NativeThreadDescription {
    pub no: u64,
    pub last_modified: u32
}

#[derive(RustcDecodable, Debug)]
pub struct NativeThreadSection {
    pub page: u8,
    pub threads: Vec<NativeThreadDescription>
}

#[derive(RustcDecodable, Debug)]
pub struct NativeCatalogSection {
    pub page: u8,
    pub threads: Vec<NativePost>
}

#[derive(RustcDecodable, Debug)]
pub struct NativeBoard {
    pub board: String,
    pub title: String,
    pub ws_board: u8,
    pub per_page: u8,
    pub pages: u8,
    //pub is_archived: Option<u8>
}

#[derive(RustcDecodable, Debug)]
pub struct NativeBoards {
    pub boards: Vec<NativeBoard>
}

impl FourchanFile {
    fn url(&self, url_type: UrlType) -> String {
        let mut subdomain = "t";
        let mut ext = ".jpg".to_owned();
        let mut filename = self._renamed.clone();

        match url_type {
            UrlType::Thumb => {
                filename = filename + "s";
            },
            UrlType::Full => {
                subdomain = "i";
                ext = self._extension.clone();
            },
        }

        format!("https://{}.4cdn.org/{}/{}{}", subdomain, self._board, filename, ext)
    }

    fn filename(&self, filename_type: FilenameType) -> &String {
        match filename_type {
            FilenameType::Renamed => &self._renamed,
            FilenameType::Original => &self._filename,
        }
    }

    fn extension(&self) -> &String {
        &self._extension
    }

    fn size(&self) -> &u32 {
        &self._size
    }

    fn md5(&self) -> &String {
        &self._md5
    }

    fn width(&self) -> &u16 {
        &self._w
    }

    fn height(&self) -> &u16 {
        &self._h
    }

    fn thumb_width(&self) -> &u8 {
        &self._thumb_w
    }

    fn thumb_height(&self) -> &u8 {
        &self._thumb_h
    }

    fn deleted(&self) -> &bool {
        &self._deleted
    }

    fn spoiler(&self) -> SpoilerType {
        if self._spoiler {
            SpoilerType::Yes(self._custom_spoiler)
        } else {
            SpoilerType::No
        }
    }
}

impl HasUrl for FourchanPost {
    fn url(&self) -> String {
        format!("https://boards.4chan.org/{}/thread/{}#p{}", self._board, match self._op {
            OpStatus::No(op) => op,
            OpStatus::Yes => self._no,
        }, self._no)
    }
}

impl FourchanPost {
    pub fn board(&self) -> &str {
        &self._board
    }

    pub fn no(&self) -> &u64 {
        &self._no
    }

    pub fn op(&self) -> &OpStatus {
        &self._op
    }

    pub fn sticky(&self) -> &bool {
        &self._sticky
    }

    pub fn closed(&self) -> &bool {
        &self._closed
    }

    pub fn timestamp(&self) -> &u32 {
        &self._timestamp
    }

    pub fn name(&self) -> &Option<String> {
        &self._name
    }

    pub fn tripcode(&self) -> &Option<String> {
        &self._trip
    }

    pub fn id(&self) -> &Option<String> {
        &self._id
    }

    pub fn capcode(&self) -> &Option<String> {
        &self._capcode
    }

    pub fn country(&self) -> &Option<[char; 2]> {
        &self._country
    }

    pub fn subject(&self) -> &String {
        &self._subject
    }

    pub fn comment(&self) -> &String {
        &self._comment
    }

    pub fn files(&self) -> &Vec<FourchanFile> {
        &self._files
    }
}

impl HasUrl for FourchanThread {
    fn url(&self) -> String {
        format!("https://boards.4chan.org/{}/thread/{}", self._board, self._no)
    }
}

impl FourchanThread {
    pub fn no(&self) -> &u64 {
        &self._no
    }

    pub fn board(&self) -> &str {
        &self._board
    }

    pub fn last_modified(&self) -> &Option<u32> {
        &self._last_modified
    }

    pub fn posts(&mut self) -> Result<Vec<FourchanPost>, ChanError> {
        if self._empty {
            self.update()
        } else {
            Ok(self._posts.clone())
        }
    }

    pub fn update(&mut self) -> Result<Vec<FourchanPost>, ChanError> {
        let url = "http://a.4cdn.org/".to_owned() + &self._board + "/thread/" + &self._no.to_string() + ".json";
        match download(&url) {
            Err(e) => Err(ChanError::HTTP(e)),
            Ok((res, timestamp)) => match json::decode::<NativeThread>(&res) {
                Err(e) => Err(ChanError::JSON(e)),
                Ok(parsed) => {
                    let mut posts: Vec<FourchanPost> = Vec::with_capacity(parsed.posts.len() as usize);
                    for post in parsed.posts {
                        posts.push(setup_post(self._board.clone(), post));
                    }
                    self._empty = false;
                    self._posts = posts.clone();
                    Ok(posts)
                },
            },
        }
    }
}

impl HasUrl for FourchanBoard {
    fn url(&self) -> String {
        format!("https://boards.4chan.org/{}/", self._shortcode)
    }
}

impl FourchanBoard {
    pub fn shortcode(&self) -> &str {
        &self._shortcode
    }

    pub fn title(&self) -> &str {
        &self._title
    }

    pub fn work_safe(&self) -> &bool {
        &self._work_safe
    }

    pub fn per_page(&self) -> &u8 {
        &self._per_page
    }

    pub fn pages(&self) -> &u8 {
        &self._pages
    }

    pub fn update(&mut self) -> Result<Vec<FourchanThread>, ChanError> {
        let url = "http://a.4cdn.org/".to_owned() + &self._shortcode + "/";
        if self._using_catalog { // TODO: use NativePost for both and drop all the custom threads.json structs
            match download(&(url + "catalog.json")) {
                Err(e) => Err(ChanError::HTTP(e)),
                Ok((content, timestamp)) => match json::decode::<Vec<NativeCatalogSection>>(&content) {
                    Err(e) => Err(ChanError::JSON(e)),
                    Ok(parsed) => {
                        let mut threads: Vec<FourchanThread> = Vec::with_capacity((self._per_page * self._pages) as usize);
                        for page in parsed {
                            for thread in page.threads {
                                let thread_clone = thread.clone();
                                threads.push(FourchanThread {
                                    _no: thread.no,
                                    _last_modified: None,
                                    _empty: true,
                                    _board: self._shortcode.clone(),
                                    _op: Some(setup_post(self._shortcode.clone(), thread_clone)),
                                    _posts: Vec::new(),
                                });
                            }
                        }
                        self._empty = false;
                        self._threads = threads.clone();
                        Ok(threads)
                    },
                },
            }
        } else {
            match download(&(url + "threads.json")) {
                Err(e) => Err(ChanError::HTTP(e)),
                Ok((content, timestamp)) => match json::decode::<Vec<NativeThreadSection>>(&content) {
                    Err(e) => Err(ChanError::JSON(e)),
                    Ok(parsed) => {
                        let mut threads: Vec<FourchanThread> = Vec::with_capacity((self._per_page * self._pages) as usize);
                        for page in parsed {
                            for thread in page.threads {
                                threads.push(FourchanThread {
                                    _no: thread.no,
                                    _last_modified: None,
                                    _empty: true,
                                    _board: self._shortcode.clone(),
                                    _op: None,
                                    _posts: Vec::new(),
                                });
                            }
                        }
                        self._empty = false;
                        self._threads = threads.clone();
                        Ok(threads)
                    },
                },
            }
        }
    }

    pub fn threads(&mut self) -> Result<Vec<FourchanThread>, ChanError> {
        if self._empty {
            self.update()
        } else {
            Ok(self._threads.clone())
        }
    }

    //pub fn stickied(&mut self) -> Result<Vec<FourchanThread>, DecoderError> {
    //}

    /*pub fn threads(&self) -> Result<&Vec<FourchanThread>, DecoderError> {
        let parsed = json::decode::<Vec<NativeThreadSection>>(&(download(&("http://a.4cdn.org/".to_owned() + &self._shortcode + "/threads.json")).unwrap()));
        if self._empty {
            match parsed {
                Err(e) => Err(e),
                Ok(res) => {
                    let mut wrapped: Vec<FourchanThread> = Vec::with_capacity((self._per_page * self._pages) as usize);
                    for thread_section in res {
                        for thread_description in thread_section.threads {
                            wrapped.push(FourchanThread {
                                _no: thread_description.no,
                                _last_modified: None,
                                _posts: None,
                                _new: true,
                                _board: self._shortcode.clone(),
                                _op: None,
                            });
                        }
                    }   
                    Ok(&self._threads)
                },
            }
        }
        else {
            Ok(&self._threads)
        }
    }*/
}

impl HasUrl for Fourchan {
    fn url(&self) -> String {
        "https://www.4chan.org/".to_owned()
    }
}

impl Fourchan {
    pub fn new() -> Fourchan {
        Fourchan {
            _boards: BTreeMap::new()
        }
    }

    pub fn update(&mut self) -> Result<BTreeMap<String, FourchanBoard>, ChanError> {
        match download("http://a.4cdn.org/boards.json") {
            Err(e) => Err(ChanError::HTTP(e)),
            Ok((res, timestamp)) => match json::decode::<NativeBoards>(&res) {
                Err(e) => Err(ChanError::JSON(e)),
                Ok(parsed) => {
                    let mut boards = BTreeMap::new();
                    for board in parsed.boards {
                        boards.insert(board.board.clone(), FourchanBoard {
                            _shortcode: board.board,
                            _title: board.title,
                            _work_safe: board.ws_board > 0,
                            _per_page: board.per_page,
                            _pages: board.pages,
                            _using_catalog: false,
                            _empty: true,
                            _threads: Vec::new(),
                        });
                    }
                    self._boards = boards.clone();
                    Ok(boards)
                },
            },
        }
    }

    pub fn board(&mut self, shortcode: &str) -> Result<FourchanBoard, ChanError> {
        match self.boards() {
            Err(e) => Err(e),
            Ok(boards_map) => {
                let boards_iter: IntoIter<String, FourchanBoard> = boards_map.into_iter();
                let mut board_res: Result<FourchanBoard, ChanError> = Err(ChanError::NonexistantBoard(shortcode.to_owned()));
                for (code, board_obj) in boards_iter {
                    if code == shortcode {
                        board_res = Ok(board_obj)
                    }
                }
                board_res
            },
        }
    }

    pub fn boards(&mut self) -> Result<BTreeMap<String, FourchanBoard>, ChanError> { // from Chan trait (not shown here)
        if self._boards.is_empty() {
            self.update()
        } else {
            Ok(self._boards.clone())
        }
    }
}

#[test]
fn parse_post() {
    let mut g = Fourchan::new().board("g").unwrap();
    let mut threads = g.threads().unwrap();
    let ref thread = threads[0].posts().unwrap()[0];
    //println!("{}", thread.comment());
    //println!("{}", thread.files()[0].url(UrlType::Full));
}
