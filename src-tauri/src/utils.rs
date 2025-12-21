use std::collections::HashMap;
use std::ffi::OsStr;
use everything_sdk::{global, EverythingError, RequestFlags, SortType};
use image::{ImageEncoder, RgbaImage};
use std::mem;
use std::mem::MaybeUninit;
use std::path::PathBuf;
use std::ptr::addr_of_mut;
use std::sync::{Arc, LazyLock, Mutex};
use image::codecs::png::PngEncoder;
use image::ExtendedColorType::Rgba8;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::Gdi::{DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::UI::Shell::{ExtractIconExW, SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use windows::Win32::UI::WindowsAndMessaging::{GetIconInfo, HICON, ICONINFO};


#[derive(Debug)]
pub struct FileInfo {
    size:usize,
    name: String,
    path:PathBuf

}


impl FileInfo {
    fn new(path:PathBuf,size:usize) -> FileInfo {
        let name = match path.file_name() {
            Some(name) => name.to_string_lossy().into_owned(),
            None => "UnknownFile".to_string()
        };

        Self {
            size,
            name,
            path: path.to_path_buf()
        }

    }
    
    pub fn get_size(&self) -> usize {
        self.size
    }
    
    pub fn get_name(&self) ->String {
        self.name.clone()
    }
    
    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn is_file(&self) -> bool {
        self.path.is_file()
    }
}

pub struct  EverythingHelper{
    max:usize,
}

impl Default for EverythingHelper {
    fn default() -> Self {
        Self { max:20 }
    }
}
impl EverythingHelper {


    pub fn set_max(mut self, max: usize) -> Self {
        self.max = max;
        self
    }
    pub async fn check_everything_server() -> Result<bool, EverythingError> {
        global().lock().await.is_db_loaded()
    }
    pub async fn query(&self, input:&str) -> Vec<FileInfo> {
        let mut everything = global().lock().await;

        // All other things are consistent with the sync version. (expect searcher.query())

        match everything.is_db_loaded() {
            Ok(false) => panic!("The Everything database has not been fully loaded now."),
            Err(EverythingError::Ipc) => panic!("Everything is required to run in the background."),
            _ => {
                let mut searcher = everything.searcher();
                let query_str = format!(
                    "{} !path:*$RECYCLE.BIN* !path:C:\\Windows\\*",
                    input.to_string()
                );
                searcher.set_search(query_str.as_str());
                searcher
                    .set_request_flags(
                        RequestFlags::EVERYTHING_REQUEST_FILE_NAME
                            | RequestFlags::EVERYTHING_REQUEST_PATH
                            | RequestFlags::EVERYTHING_REQUEST_SIZE
                            | RequestFlags::EVERYTHING_REQUEST_RUN_COUNT,
                    )
                    .set_max(self.max as u32)
                    .set_sort(SortType::EVERYTHING_SORT_FILE_LIST_FILENAME_ASCENDING);

                //assert_eq!(searcher.get_match_case(), false);
                dbg!(&searcher.get_search());
                // Send IPC query in Async, await for the result. So we are _unblocking_ now.
                // Some heavy query (like search single 'a') may take a lot of time in IPC data transfer.
                // So during this time, tokio goes to deal with other tasks.
                // When the IPC done, it will yield back for us.
                let results = searcher.query().await;

                //let visible_num_results = dbg!(results.num());
                //let total_num_results = dbg!(results.total());


                // let is_attr_flag_set =
                //     dbg!(results.request_flags()).contains(RequestFlags::EVERYTHING_REQUEST_ATTRIBUTES);
                // assert!(!is_attr_flag_set);




                let res:Vec<FileInfo> = results.iter().map(|item|{
                    FileInfo::new( item.filepath().unwrap(),item.size().unwrap() as usize )
                } ).collect();
                drop(results);

                return res;
                // let run_count = results
                //     .at(2)
                //     .expect("I'm pretty sure there are at least 3 results.")
                //     .run_count()
                //     .unwrap();
                // println!("Run Count for Item[2]: `{}`", run_count);

                // Remember, because of global variables, there can only be one `everything`, `searcher`
                // and `results` at any time during the entire program lifetime.

                // Even being in Async mode, it doesn't change this thing.

                drop(results);
                // searcher.set_search("cargo");
                // let _results = searcher.query();
                // The `searcher` will be dropped here as out of scope.
            }
        }
    }
}


static ICON_CACHE:LazyLock<Arc<Mutex<HashMap<String,RgbaImage>>>> = LazyLock::new(|| {Arc::new(Mutex::new(HashMap::new()))});

pub struct IconExtractor{
    cache: Arc<Mutex<HashMap<String,RgbaImage>>>
}

impl Default for IconExtractor {
    fn default() -> Self {
        Self {
            cache: ICON_CACHE.clone()
        }
    }
}
impl IconExtractor {

    pub fn get_icon(&self,path:&PathBuf) -> Option<RgbaImage>{
        let mut key = path.file_name().unwrap().to_str().unwrap().to_string();


        let ext = path.extension().unwrap_or(OsStr::new("empty")).to_str().unwrap().to_lowercase();
        if !(ext == "exe" || ext == "lnk" || ext == "ico") {
            key = ext.to_string();
        }
        let lock = self.cache.lock().unwrap();
        if let Some(data) = lock.get(&key) {
            Some(data.clone())
        }else {
            drop(lock);
            let data=  self._get_icon(path);
            if let Some(data) = data {
                let mut lock = self.cache.lock().unwrap();
                lock.insert(key,data.clone());
                drop(lock);
                Some(data)
            }else{
                None
            }
        }
    }
    fn _get_icon(&self,path:&PathBuf) -> Option<RgbaImage>{
        let a = path.extension();
        if let Some(ext) = a {
            if let Some(ext) = ext.to_str() {

                if ext == "exe"{
                    if let Some(hicon) = self.get_icon_from_exe(&path) {
                        self.hicon_to_png(hicon)
                    }else {None}


                }else {
                    dbg!(path,path.is_dir());
                    if let Some(hicon) = self.get_icon_from_file(&path) {
                        self.hicon_to_png(hicon)
                    }else {None}
                }
            }else {
                None
            }
        }else {
            None
        }
    }
    fn get_icon_from_exe(&self, path: &PathBuf) -> Option<HICON>{
        unsafe {
            // 将路径转换为宽字符格式
            let path = path.to_str().unwrap();

            let wide_path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
            let wide_path_ptr = PCWSTR(wide_path.as_ptr());

            // 定义返回的大图标和小图标
            let mut large_icon = HICON::default();
            let mut small_icon = HICON::default();

            // 调用 API 提取图标
            let result = ExtractIconExW(
                wide_path_ptr,
                0,
                Some(&mut large_icon),
                Some(&mut small_icon),
                1, // 提取一个图标
            );

            if result > 0 {
                Some(large_icon) // 返回大图标句柄
            } else {
                None
            }
        }
    }

    fn get_icon_from_file(&self,path:&PathBuf) -> Option<HICON>{
        let mut file_info = SHFILEINFOW::default();


        unsafe {
            let path = path.to_str().unwrap();
            let wide_path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
            let wide_path_ptr = PCWSTR(wide_path.as_ptr());
            SHGetFileInfoW(
                wide_path_ptr,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                Some(&mut file_info),
                size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON,
            );
        }
        let hicon: HICON = file_info.hIcon;
        Some(hicon)
    }

    fn hicon_to_png(&self, hicon: HICON) -> Option<RgbaImage>{
        unsafe {
            // 获取图标信息
            let mut ii = ICONINFO::default();
            let _ = GetIconInfo(hicon, &mut ii);



            let bitmap_size_i32 = i32::try_from(size_of::<BITMAP>()).unwrap();
            let biheader_size_u32 = u32::try_from(size_of::<BITMAPINFOHEADER>()).unwrap();

            let mut bitmap: MaybeUninit<BITMAP> = MaybeUninit::uninit();
            let result = GetObjectW(
                ii.hbmColor.into(),
                bitmap_size_i32,
                Some(bitmap.as_mut_ptr().cast()),
            );
            if (result != bitmap_size_i32){
                let _ = ii.hbmColor;
                let _ = ii.hbmMask;
                let _ = bitmap;

                return  None
            }
            let bitmap = bitmap.assume_init_ref();


            // ico info
            let width_u32 = u32::try_from(bitmap.bmWidth).unwrap();
            let height_u32 = u32::try_from(bitmap.bmHeight).unwrap();
            let width_usize = usize::try_from(bitmap.bmWidth).unwrap();
            let height_usize = usize::try_from(bitmap.bmHeight).unwrap();

            // cal buffer
            let buf_size = width_usize
                .checked_mul(height_usize)
                .and_then(|size| size.checked_mul(4))
                .unwrap();
            let mut buf: Vec<u8> = Vec::with_capacity(buf_size);
            dbg!(buf.capacity());


            let dc = GetDC(Some(HWND::default()));
            assert_ne!(dc, HDC::default());


            let mut bitmap_info = BITMAPINFOHEADER {
                biSize: biheader_size_u32,
                biWidth: bitmap.bmWidth,
                biHeight: -bitmap.bmHeight,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            };

            let result = GetDIBits(
                dc,
                ii.hbmColor,
                0,
                height_u32,
                Some(buf.as_mut_ptr().cast()),
                addr_of_mut!(bitmap_info).cast(),
                DIB_RGB_COLORS,
            );

            assert_eq!(result, bitmap.bmHeight);
            buf.set_len(buf.capacity());

            let result = ReleaseDC(Some(HWND::default()), dc);
            assert_eq!(result, 1);
            DeleteObject(ii.hbmColor.into()).unwrap();
            DeleteObject(ii.hbmMask.into()).unwrap();


            // swap B R
            for chunk in buf.chunks_exact_mut(4) {
                let [b, _, r, _] = chunk else { unreachable!() };
                mem::swap(b, r);
            }

            RgbaImage::from_vec(width_u32, height_u32, buf)
        }

    }
}


pub fn to_base64(data:RgbaImage) -> String {
    let mut buffer = Vec::new();
    let png_encoder = PngEncoder::new(&mut buffer);


    let _ = png_encoder.write_image(data.as_raw(), data.width(),data.height(),Rgba8);

    use base64::{engine::general_purpose, Engine as _, };

    let b64 = general_purpose::STANDARD.encode(buffer);
    b64
}