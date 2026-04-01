/// <reference path="./.sst/platform/config.d.ts" />

/**
 * Plantastic infrastructure — SST v3
 *
 * Deploys the Rust API binary to Lambda with a Function URL.
 * The binary auto-detects Lambda mode via AWS_LAMBDA_RUNTIME_API env var.
 */
export default $config({
  app(input) {
    return {
      name: "plantastic",
      removal: input?.stage === "production" ? "retain" : "remove",
      protect: ["production"].includes(input?.stage ?? ""),
      home: "aws",
      providers: {
        aws: {
          region: "us-west-2",
        },
      },
    };
  },
  async run() {
    // Secrets from SSM parameter store (set per stage)
    const databaseUrl = new sst.Secret("DatabaseUrl");
    const anthropicKey = new sst.Secret("AnthropicApiKey");

    // S3 bucket for scan uploads and generated artifacts
    const uploads = new sst.aws.Bucket("Uploads", {
      cors: {
        allowOrigins: ["*"],
        allowMethods: ["GET", "PUT", "POST"],
        allowHeaders: ["*"],
        maxAge: "1 hour",
      },
    });

    const api = new sst.aws.Function("Api", {
      runtime: "provided.al2023",
      architecture: "arm64",
      handler: "bootstrap",
      bundle: "target/lambda/plantastic-api",
      memory: "256 MB",
      timeout: "30 seconds",
      link: [uploads],
      url: {
        authorization: "none",
        invokeMode: "RESPONSE_STREAM",
      },
      environment: {
        DATABASE_URL: databaseUrl.value,
        ANTHROPIC_API_KEY: anthropicKey.value,
        S3_BUCKET: uploads.name,
        RUST_LOG: "plantastic_api=info,pt_repo=info,warn",
        HOME: "/tmp",
        BAML_LIBRARY_PATH:
          "/var/task/libbaml_cffi-aarch64-unknown-linux-gnu.so",
      },
    });

    return {
      apiUrl: api.url,
      bucketName: uploads.name,
    };
  },
});
