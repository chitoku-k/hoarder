group "default" {
    targets = ["api"]
}

target "api" {
    context = "./api"
    tags = ["container.chitoku.jp/chitoku-k/hoarder/api"]
}
