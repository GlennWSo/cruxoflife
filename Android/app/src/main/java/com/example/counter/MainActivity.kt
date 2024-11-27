package com.example.counter

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.counter.shared_types.Event
import com.example.counter.ui.theme.CounterTheme
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            CounterTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    View()
                }
            }
        }
    }
}

@Composable
fun View(core: Core = viewModel()) {
    val coroutineScope = rememberCoroutineScope()
    Canvas(modifier = Modifier.fillMaxSize().background(color=Color.Green)){

        val canvasQuadrantSize = size / 2F
        val h = size.height
        val w = size.width
        val cellSize = 30f

        val nCols = (w / cellSize ).roundToInt()
        val nRows = (h / cellSize ).roundToInt()

        repeat(nCols + 1) {
            drawLine(
                strokeWidth = 3f,
                color = Color.Black,
                start = Offset(x = cellSize * it, y = 0f),
                end = Offset(x = cellSize * it, y = h),
                colorFilter = ColorFilter.tint(Color.Black)
            )
        }
        repeat(nRows + 1) {
            drawLine(
                strokeWidth = 3f,
                color = Color.Black,
                start = Offset(y = cellSize * it, x = 0f),
                end = Offset(y = cellSize * it, x = h),
                colorFilter = ColorFilter.tint(Color.Black)
            )
        }

        // Draw a rectangle
        // drawRect(color = Color.Magenta, size = canvasQuadrantSize)
        // Draw a circle
        // drawCircle(color = Color.Cyan, radius = 400f)
    }

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = Modifier.fillMaxSize(),
    ) {
        Row(
            horizontalArrangement = Arrangement.Center,
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth().background(color = Color.White)) {
            Text(text = "Crux Game of Life", fontSize = 30.sp, modifier = Modifier.padding(10.dp).background(color = Color.White))

        }

        // Text(text = "Rust Core, Kotlin Shell (Jetpack Compose)", modifier = Modifier.padding(10.dp))
        Spacer(
            modifier = Modifier.weight(1f)
        )
        Row( modifier = Modifier.fillMaxWidth().background(color=Color.White),
            horizontalArrangement = Arrangement.Center
            ) {
            Button(
                modifier = Modifier.padding(10.dp),
                onClick = {
                    coroutineScope.launch { core.update(Event.Decrement()) }
                }, colors = ButtonDefaults.buttonColors(
                    containerColor = Color.hsl(44F, 1F, 0.77F)
                )
            ) { Text(text = "Decrement", color = Color.DarkGray) }
            Button(
                modifier = Modifier.padding(10.dp),
                onClick = {
                    coroutineScope.launch { core.update(Event.Increment()) }
                }, colors = ButtonDefaults.buttonColors(
                    containerColor = Color.hsl(348F, 0.86F, 0.61F)
                )
            ) { Text(text = "Increment", color = Color.White) }
        }
    }
}

@Preview(showBackground = true)
@Composable
fun DefaultPreview() {
    CounterTheme { View() }
}
