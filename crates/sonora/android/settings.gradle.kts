pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
        // `rustls-platform-verifier` ships its Kotlin certificate checker from its own
        // Maven-shaped branch on GitHub; the version is pinned by Cargo.lock below.
        maven {
            url = uri("https://github.com/rustls/rustls-platform-verifier/raw/maven-archive/android-release-support/maven/")
        }
    }
}

rootProject.name = "sonora"
include(":app")
