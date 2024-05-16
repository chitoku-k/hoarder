group "default" {
    targets = ["api", "ui"]
}

target "api" {
    context = "./api"
    tags = ["container.chitoku.jp/chitoku-k/hoarder/api"]
}

target "ui" {
    context = "./ui"
    contexts = {
        schema = "./schema"
    }
    tags = ["container.chitoku.jp/chitoku-k/hoarder/ui"]
}
