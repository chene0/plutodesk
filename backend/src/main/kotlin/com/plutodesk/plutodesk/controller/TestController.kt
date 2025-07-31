package com.plutodesk.plutodesk.controller

import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.RestController

@RestController
@RequestMapping("/api")
class TestController {

    @GetMapping("/hello")
    fun hello(): Map<String, String> {
        return mapOf("message" to "Hello from PlutoDesk Backend!")
    }

    @GetMapping("/health")
    fun health(): Map<String, String> {
        return mapOf("status" to "OK", "service" to "PlutoDesk API")
    }
}
