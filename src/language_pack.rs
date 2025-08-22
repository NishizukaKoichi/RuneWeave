use anyhow::Result;
use std::path::Path;
use tera::{Context as TeraContext, Tera};

use crate::verify::{Language, Service};

pub trait LanguagePack {
    fn render_service(
        &self,
        service: &Service,
        out_dir: &Path,
        tera: &mut Tera,
        ctx: &TeraContext,
    ) -> Result<()>;

    fn register_templates(&self, tera: &mut Tera) -> Result<()>;
}

pub struct RustPack;
pub struct NodePack;
pub struct PythonPack;
pub struct GoPack;
pub struct JavaPack;

impl LanguagePack for RustPack {
    fn register_templates(&self, tera: &mut Tera) -> Result<()> {
        // Rust Cargo.toml template
        tera.add_raw_template(
            "rust-cargo.toml",
            r#"[package]
name = "{{ service_name }}"
version = "0.1.0"
edition = "2021"
rust-version = "{{ rust_version }}"

[dependencies]
{%- if framework == "actix" %}
actix-web = "4"
{%- elif framework == "worker" %}
worker = "0.6"
{%- endif %}
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
anyhow = "1.0"
"#,
        )?;

        // Rust main.rs for Actix
        tera.add_raw_template(
            "rust-actix-main.rs",
            r#"use actix_web::{web, App, HttpResponse, HttpServer};
use tracing::info;

async fn healthz() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting {{ service_name }} server on 0.0.0.0:8080");
    
    HttpServer::new(|| {
        App::new()
            .route("/healthz", web::get().to(healthz))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
"#,
        )?;

        // Rust lib.rs for Workers
        tera.add_raw_template(
            "rust-worker-lib.rs",
            r#"use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/healthz", |_, _| {
            Response::ok(serde_json::json!({
                "status": "healthy"
            }).to_string())
        })
        .run(req, env)
        .await
}
"#,
        )?;

        Ok(())
    }

    fn render_service(
        &self,
        service: &Service,
        out_dir: &Path,
        tera: &mut Tera,
        ctx: &TeraContext,
    ) -> Result<()> {
        let service_dir = out_dir.join("services").join(&service.name);
        std::fs::create_dir_all(&service_dir)?;

        let mut service_ctx = ctx.clone();
        service_ctx.insert("service_name", &service.name);
        service_ctx.insert(
            "framework",
            &service.framework.as_deref().unwrap_or("actix"),
        );

        // Add rust_version from toolchain config
        if let Some(toolchain) = ctx.get("toolchain") {
            if let Some(rust_toolchain) = toolchain.get("rust") {
                if let Some(version) = rust_toolchain.get("version") {
                    service_ctx.insert("rust_version", version);
                }
            }
        }

        // Cargo.toml
        let content = tera.render("rust-cargo.toml", &service_ctx)?;
        std::fs::write(service_dir.join("Cargo.toml"), content)?;

        // src/main.rs or src/lib.rs
        let src_dir = service_dir.join("src");
        std::fs::create_dir_all(&src_dir)?;

        match service.framework.as_deref() {
            Some("worker") => {
                let content = tera.render("rust-worker-lib.rs", &service_ctx)?;
                std::fs::write(src_dir.join("lib.rs"), content)?;
            }
            _ => {
                let content = tera.render("rust-actix-main.rs", &service_ctx)?;
                std::fs::write(src_dir.join("main.rs"), content)?;
            }
        }

        Ok(())
    }
}

impl LanguagePack for NodePack {
    fn register_templates(&self, tera: &mut Tera) -> Result<()> {
        // Node package.json
        tera.add_raw_template(
            "node-package.json",
            r#"{
  "name": "{{ service_name }}",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "tsx watch src/index.ts",
    "build": "tsc",
    "start": "node dist/index.js",
    "test": "vitest run",
    "lint": "eslint src"
  },
  "dependencies": {
{%- if framework == "fastify" %}
    "fastify": "^4.0.0",
{%- elif framework == "hono" and runtime == "cloudflare" %}
    "@cloudflare/workers-types": "^4.0.0",
    "hono": "^3.0.0",
{%- endif %}
    "zod": "^3.0.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "eslint": "^8.0.0",
    "tsx": "^4.0.0",
    "typescript": "^5.0.0",
    "vitest": "^1.0.0"
  }
}
"#,
        )?;

        // TypeScript index.ts
        tera.add_raw_template(
            "node-index.ts",
            r#"{% if framework == "fastify" %}
import Fastify from 'fastify'

const app = Fastify({ logger: true })

app.get('/healthz', async () => {
  return { status: 'healthy' }
})

const start = async () => {
  try {
    await app.listen({ port: 3000, host: '0.0.0.0' })
  } catch (err) {
    app.log.error(err)
    process.exit(1)
  }
}

start()
{% elif framework == "hono" and runtime == "cloudflare" %}
import { Hono } from 'hono'

const app = new Hono()

app.get('/healthz', (c) => {
  return c.json({ status: 'healthy' })
})

export default app
{% endif %}
"#,
        )?;

        // tsconfig.json
        tera.add_raw_template(
            "node-tsconfig.json",
            r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "node",
    "esModuleInterop": true,
    "strict": true,
    "skipLibCheck": true,
    "outDir": "dist",
    "rootDir": "src"
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
"#,
        )?;

        Ok(())
    }

    fn render_service(
        &self,
        service: &Service,
        out_dir: &Path,
        tera: &mut Tera,
        ctx: &TeraContext,
    ) -> Result<()> {
        let service_dir = out_dir.join("services").join(&service.name);
        std::fs::create_dir_all(&service_dir)?;

        let mut service_ctx = ctx.clone();
        service_ctx.insert("service_name", &service.name);
        service_ctx.insert(
            "framework",
            &service.framework.as_deref().unwrap_or("fastify"),
        );
        service_ctx.insert("runtime", &service.runtime.as_deref().unwrap_or("node"));

        // package.json
        let content = tera.render("node-package.json", &service_ctx)?;
        std::fs::write(service_dir.join("package.json"), content)?;

        // tsconfig.json
        let content = tera.render("node-tsconfig.json", &service_ctx)?;
        std::fs::write(service_dir.join("tsconfig.json"), content)?;

        // src/index.ts
        let src_dir = service_dir.join("src");
        std::fs::create_dir_all(&src_dir)?;
        let content = tera.render("node-index.ts", &service_ctx)?;
        std::fs::write(src_dir.join("index.ts"), content)?;

        Ok(())
    }
}

impl LanguagePack for PythonPack {
    fn register_templates(&self, tera: &mut Tera) -> Result<()> {
        // Python pyproject.toml
        tera.add_raw_template(
            "python-pyproject.toml",
            r#"[tool.poetry]
name = "{{ service_name }}"
version = "0.1.0"
description = ""

[tool.poetry.dependencies]
python = "^{{ python_version }}"
{%- if framework == "fastapi" %}
fastapi = "^0.100.0"
uvicorn = "^0.30.0"
{%- endif %}
pydantic = "^2.0.0"

[tool.poetry.group.dev.dependencies]
pytest = "^8.0.0"
ruff = "^0.5.0"

[build-system]
requires = ["poetry-core"]
build-backend = "poetry.core.masonry.api"
"#,
        )?;

        // Python main.py
        tera.add_raw_template(
            "python-main.py",
            r#"{% if framework == "fastapi" %}
from fastapi import FastAPI
from pydantic import BaseModel

app = FastAPI()

class HealthResponse(BaseModel):
    status: str

@app.get("/healthz", response_model=HealthResponse)
def health_check():
    return {"status": "healthy"}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
{% else %}
def main():
    print("{{ service_name }} started")

if __name__ == "__main__":
    main()
{% endif %}
"#,
        )?;

        Ok(())
    }

    fn render_service(
        &self,
        service: &Service,
        out_dir: &Path,
        tera: &mut Tera,
        ctx: &TeraContext,
    ) -> Result<()> {
        let service_dir = out_dir.join("services").join(&service.name);
        std::fs::create_dir_all(&service_dir)?;

        let mut service_ctx = ctx.clone();
        service_ctx.insert("service_name", &service.name);
        service_ctx.insert("framework", &service.framework.as_deref().unwrap_or("none"));
        service_ctx.insert("python_version", &"3.12");

        // pyproject.toml
        let content = tera.render("python-pyproject.toml", &service_ctx)?;
        std::fs::write(service_dir.join("pyproject.toml"), content)?;

        // src/main.py
        let src_dir = service_dir.join("src");
        std::fs::create_dir_all(&src_dir)?;
        let content = tera.render("python-main.py", &service_ctx)?;
        std::fs::write(src_dir.join("main.py"), content)?;

        // tests/__init__.py
        let tests_dir = service_dir.join("tests");
        std::fs::create_dir_all(&tests_dir)?;
        std::fs::write(tests_dir.join("__init__.py"), "")?;

        Ok(())
    }
}

impl LanguagePack for GoPack {
    fn register_templates(&self, tera: &mut Tera) -> Result<()> {
        // Go go.mod
        tera.add_raw_template(
            "go-mod",
            r#"module {{ service_name }}

go {{ go_version }}

require (
{%- if framework == "gin" %}
    github.com/gin-gonic/gin v1.9.1
{%- elif framework == "fiber" %}
    github.com/gofiber/fiber/v2 v2.52.0
{%- endif %}
)
"#,
        )?;

        // Go main.go
        tera.add_raw_template(
            "go-main.go",
            r#"package main

{% if framework == "gin" %}
import (
    "net/http"
    "github.com/gin-gonic/gin"
)

func main() {
    r := gin.Default()
    
    r.GET("/healthz", func(c *gin.Context) {
        c.JSON(http.StatusOK, gin.H{
            "status": "healthy",
        })
    })
    
    r.Run(":8080")
}
{% elif framework == "fiber" %}
import (
    "github.com/gofiber/fiber/v2"
)

func main() {
    app := fiber.New()
    
    app.Get("/healthz", func(c *fiber.Ctx) error {
        return c.JSON(fiber.Map{
            "status": "healthy",
        })
    })
    
    app.Listen(":8080")
}
{% else %}
import (
    "fmt"
    "net/http"
)

func healthHandler(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Content-Type", "application/json")
    w.Write([]byte(`{"status":"healthy"}`))
}

func main() {
    http.HandleFunc("/healthz", healthHandler)
    fmt.Println("Server starting on :8080")
    http.ListenAndServe(":8080", nil)
}
{% endif %}
"#,
        )?;

        Ok(())
    }

    fn render_service(
        &self,
        service: &Service,
        out_dir: &Path,
        tera: &mut Tera,
        ctx: &TeraContext,
    ) -> Result<()> {
        let service_dir = out_dir.join("services").join(&service.name);
        std::fs::create_dir_all(&service_dir)?;

        let mut service_ctx = ctx.clone();
        service_ctx.insert("service_name", &service.name);
        service_ctx.insert(
            "framework",
            &service.framework.as_deref().unwrap_or("stdlib"),
        );
        service_ctx.insert("go_version", &"1.22");

        // go.mod
        let content = tera.render("go-mod", &service_ctx)?;
        std::fs::write(service_dir.join("go.mod"), content)?;

        // main.go
        let content = tera.render("go-main.go", &service_ctx)?;
        std::fs::write(service_dir.join("main.go"), content)?;

        Ok(())
    }
}

impl LanguagePack for JavaPack {
    fn register_templates(&self, tera: &mut Tera) -> Result<()> {
        // Java pom.xml
        tera.add_raw_template(
            "java-pom.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 
         http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    
    <groupId>com.example</groupId>
    <artifactId>{{ service_name }}</artifactId>
    <version>0.1.0</version>
    
    <properties>
        <maven.compiler.source>{{ java_version }}</maven.compiler.source>
        <maven.compiler.target>{{ java_version }}</maven.compiler.target>
        <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
    </properties>
    
    <dependencies>
{%- if framework == "spring" %}
        <dependency>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-starter-web</artifactId>
            <version>3.2.0</version>
        </dependency>
{%- endif %}
        <dependency>
            <groupId>junit</groupId>
            <artifactId>junit</artifactId>
            <version>4.13.2</version>
            <scope>test</scope>
        </dependency>
    </dependencies>
</project>
"#,
        )?;

        Ok(())
    }

    fn render_service(
        &self,
        service: &Service,
        out_dir: &Path,
        tera: &mut Tera,
        ctx: &TeraContext,
    ) -> Result<()> {
        let service_dir = out_dir.join("services").join(&service.name);
        std::fs::create_dir_all(&service_dir)?;

        let mut service_ctx = ctx.clone();
        service_ctx.insert("service_name", &service.name);
        service_ctx.insert(
            "framework",
            &service.framework.as_deref().unwrap_or("spring"),
        );
        service_ctx.insert("java_version", &"21");

        // pom.xml
        let content = tera.render("java-pom.xml", &service_ctx)?;
        std::fs::write(service_dir.join("pom.xml"), content)?;

        // Create directory structure
        let src_main = service_dir.join("src/main/java/com/example");
        std::fs::create_dir_all(&src_main)?;

        // Application.java
        let app_content = r#"package com.example;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

@SpringBootApplication
@RestController
public class Application {
    
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
    
    @GetMapping("/healthz")
    public String healthz() {
        return "{\"status\":\"healthy\"}";
    }
}
"#;
        std::fs::write(src_main.join("Application.java"), app_content)?;

        Ok(())
    }
}

pub fn get_language_pack(language: &Language) -> Box<dyn LanguagePack> {
    match language {
        Language::Rust => Box::new(RustPack),
        Language::Node => Box::new(NodePack),
        Language::Python => Box::new(PythonPack),
        Language::Go => Box::new(GoPack),
        Language::Java => Box::new(JavaPack),
        _ => Box::new(RustPack), // Default fallback
    }
}
