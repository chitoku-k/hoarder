group "default" {
    targets = ["api", "ui"]
}

target "api" {
    context = "./api"
    target = "production"
}

target "ui" {
    context = "./ui"
    contexts = {
        schema = "./schema"
    }
    target = "production"
}
