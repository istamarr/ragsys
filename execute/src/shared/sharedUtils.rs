//global or general paramter for all codes
use lazy_static::lazy_static;
use std::sync::Mutex;

pub struct UrlConnect {
    pub name: &'static str,
    pub port: &'static str,
    pub url: &'static str,
    pub path: &'static str,
}

impl UrlConnect {
    pub const AI_SERVER: UrlConnect = UrlConnect {
        name: "AI_SERVER",
        port: "9090",
        url: "http://0.0.0.0:9090",
        path: "-",
    };

    pub const LIB_SERVER: UrlConnect = UrlConnect {
        name: "LIB_SERVER",
        port: "3000",
        url: "http://0.0.0.0:3000",
        path: "-",
    };
}


pub struct Databases {
    pub name: &'static str,
    pub username: &'static str,
    pub password: &'static str,//next in db
    pub ns: &'static str,
    pub db: &'static str,
    pub port: &'static str,
    pub url: &'static str,
    pub schema: &'static str,
}
/**
* notes:
* SURREAL_DB_ prefix, sample: SURREAL_DB_APPNAME
* POSTGRES_DB_ prefix, sample: POSTGRES_DB_APPNAME
* if only prefix, it means this already used by this server
*/
impl Databases {
    pub const SURREAL_DB: Databases = Databases {
        name: "SURREAL_DB",
        username: "root",
        password: "root",
        ns: "pgd_ml_nmspace",
        db: "pgd_db",
        port: "8000",
        url: "127.0.0.1",
        schema: "-",
    };
    //mindsdb (different way to connect)
    pub const POSTGRES_DB: Databases = Databases {
        name: "POSTGRES_DB",
        username: "postgres",
        password: "admin",
        ns: "-",
        db: "transactiondb",
        port: "2345",
        url: "192.168.227.193",
        schema: "-",
    };
    pub const REDIS_DB: Databases = Databases {
        name: "REDIS_DB",
        username: "-",
        password: "-",
        ns: "-",
        db: "redis-oxide",
        port: "6379",
        // url: "redis://127.0.0.1",
        url: "127.0.0.1",
        schema: "-",
    };
}

pub struct LLM {
    pub code: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub size: &'static str,
    pub dim: &'static str,
    pub category: &'static str,
    // category: embedding, vision, tools, thinking
}

impl LLM {
    /**
    * Ai Model
    * for Ollama, embed (LLama or Tamar)
    * for Burn LM, direct inference or embed (Llama or Tamar)
    */
    //renamed from PINLM
    pub const PEDIA_AIS_LM: LLM = LLM {
        code: "ais",
        version: "latest",
        description: "AIS LM MoE",
        size: "G",
        dim: "2048",
        category: "embedding, vision, tools, thinking"
    };

    pub const PEDIA_ASIST_LM: LLM = LLM {
        code: "asist",
        version: "latest",
        description: "ASIST LM Moe",
        size: "G",
        dim: "2048",
        category: "embedding, vision, tools, thinking"
    };

    // pub const PLM: LLM = LLM {
    //     code: "PLM",
    //     version: "latest",
    //     description: "llm model PLM (Pegadaian Language Model)",
    //     size: "G",
    //     dim: "2048",
    //     category: ""
    // };

    // pub const PGLM: LLM = LLM {
    //     code: "PgLM",
    //     version: "latest",
    //     description: "llm model PgLM (Pegadaian Language Model)",
    //     size: "G",
    //     dim: "2048",
    //     category: ""
    // };

}
//note: buat loader / stream ambil dari parameter database


pub struct SizeDim {
    pub size: &'static str,
}

impl SizeDim {
    pub const SIZE_DIM_2048: SizeDim = SizeDim { size: "2048" };
    pub const SIZE_DIM_1024: SizeDim = SizeDim { size: "1024" };
    pub const SIZE_DIM_512: SizeDim = SizeDim { size: "512" };
}

pub struct MaxChar {
    pub max: &'static str,
}

impl MaxChar {
    //10.240 30.000
    pub const MAX_CHAR_IO_TXT_10240: MaxChar = MaxChar { max: "10240" };
    pub const MAX_CHAR_IO_TXT_30000: MaxChar = MaxChar { max: "30000" };
}


///SLINT-QR-CODE
pub const IMG_PATH: &str = "/img";
pub const IMG_PATH_URL: &str = "http://10.253.247.105";
pub const ALGORITHM: &str = "success";
pub const URL_SERVER: &str = "http://10.253.247.105";
pub const URL_SERVER_WEB: &str = "http://10.253.247.105/web/?branch=";
pub const IMG_NAME: &str = "pegadaian_box_logo.png";
pub const PARAM_IMG: &str = "img";
pub const PARAM_EMBED: &str = "embed";
pub const NEAREST: &str = "Nearest";
pub const TRAINGLE: &str = "Triangle";
pub const CATMULLROM: &str = "CatmullRom";
pub const GAUSSIAN: &str = "Gaussian";
pub const LANCZOS3: &str = "Lanczos3";


lazy_static! {
    pub static ref GLOBAL_ARRAY: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
}

pub struct Parameter {
    pub key: &'static str, pub value: &'static str, pub description: &'static str,
}

impl Parameter {
    pub const T2V_INDONESIA_MAN: Parameter = Parameter { key: "T2V_INDONESIA", value: "DimasNeural", description: "Text2Sound Indonesia Javanese Accents Man" };
}
