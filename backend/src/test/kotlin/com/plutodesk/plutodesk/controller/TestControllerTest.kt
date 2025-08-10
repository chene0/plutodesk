package com.plutodesk.plutodesk.controller

import org.junit.jupiter.api.Test
import org.springframework.boot.test.context.SpringBootTest
import org.springframework.boot.test.autoconfigure.web.servlet.AutoConfigureMockMvc
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.test.web.servlet.MockMvc
import org.springframework.test.web.servlet.get
import org.springframework.test.web.servlet.result.MockMvcResultMatchers.jsonPath

@SpringBootTest
@AutoConfigureMockMvc
class TestControllerTest(@Autowired val mockMvc: MockMvc) {

    @Test
    fun `actuator health endpoint returns status OK`() {
        mockMvc.get("/actuator/health")
            .andExpect { status { isOk() } }
            .andExpect { jsonPath("$.status").value("UP") }
            .andExpect { jsonPath("$.groups").isArray() }
    }
}
