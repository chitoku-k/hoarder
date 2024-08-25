variable "TAG" {
    default = "latest"
}

group "default" {
    targets = ["api", "ui"]
}

target "api" {
    context = "./api"
    tags = [
        "ghcr.io/chitoku-k/hoarder/api:latest",
        "ghcr.io/chitoku-k/hoarder/api:${TAG}",
    ]
}

target "ui" {
    context = "./ui"
    contexts = {
        schema = "./schema"
    }
    tags = [
        "ghcr.io/chitoku-k/hoarder/ui:latest",
        "ghcr.io/chitoku-k/hoarder/ui:${TAG}",
    ]
}
