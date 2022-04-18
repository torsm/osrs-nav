package de.torsm.osrsnav

import com.runemate.game.api.hybrid.location.Coordinate
import com.runemate.game.api.hybrid.region.Players
import com.runemate.game.api.script.framework.LoopingBot

class TestBot : LoopingBot() {
    override fun onStart(vararg arguments: String?) {
        setLoopDelay(5000)
    }

    override fun onLoop() {
        val local = Players.getLocal()?.position ?: return
        val result = OsrsNav.buildBetween(local, Coordinate(3270, 3228, 0))

        println(result)
        println(result?.size ?: 0)
        println()
    }
}