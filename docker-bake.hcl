group "default" {
    targets = ["api", "ui"]
}

target "api" {
    context = "./api"
}

target "ui" {
    context = "./ui"
    contexts = {
        schema = "./schema"
    }
}
